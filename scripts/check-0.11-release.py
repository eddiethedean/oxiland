#!/usr/bin/env python3
"""Fail-closed Oxiland 0.11 release qualification checker.

Derives pass/fail only from raw two-sided harness results. Rejects synthetic
passes, dirty worktrees, stale revisions, missing profiles, and profile fan-out
(identical execution_id across distinct targets).
"""

from __future__ import annotations

import hashlib
import json
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
QUAL = ROOT / "compatibility" / "qualification"
RAW = QUAL / "raw"
MATRIX = QUAL / "0.11-matrix.json"
INVENTORY = ROOT / "compatibility" / "inventory" / "redland-1.0.17-oxiland-0.11.json"
OBLIGATIONS = ROOT / "compatibility" / "inventory" / "0.11-obligations.json"
BASELINE = ROOT / "compatibility" / "baseline" / "0.11-baseline-manifest.json"
PARITY = QUAL / "0.11-parity-evidence.json"
SOAK = QUAL / "0.11-soak.json"
FUZZ = QUAL / "0.11-fuzz.json"


def fail(message: str) -> None:
    raise ValueError(message)


def load_json(path: Path) -> object:
    if not path.is_file():
        fail(f"missing required artifact: {path.relative_to(ROOT)}")
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def collect_raw_results() -> list[dict]:
    if not RAW.is_dir():
        fail("compatibility/qualification/raw/ is missing")
    results = []
    for path in sorted(RAW.glob("*.json")):
        if path.name.endswith("__index.json"):
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(data, dict):
            fail(f"raw result is not an object: {path.name}")
        data["_path"] = str(path.relative_to(ROOT))
        results.append(data)
    if not results:
        fail("no raw fixture results under compatibility/qualification/raw/")
    return results


def validate_raw(results: list[dict], matrix: dict, expected_revision: str | None) -> dict:
    required = set(matrix["required_profile_ids"])
    by_profile: dict[str, list[dict]] = defaultdict(list)
    execution_by_target: dict[str, set[str]] = defaultdict(set)

    for result in results:
        if result.get("schema_version") != 1 or result.get("milestone") != "0.11":
            fail(f"{result['_path']}: bad schema/milestone")
        if result.get("synthetic") is True:
            fail(f"{result['_path']}: synthetic evidence is forbidden")
        if result.get("skips"):
            fail(f"{result['_path']}: skips are forbidden")
        if result.get("quarantined"):
            fail(f"{result['_path']}: quarantined obligations are forbidden")
        if result.get("deviations"):
            fail(f"{result['_path']}: deviations are forbidden")
        if result.get("clean_worktree") is not True:
            fail(f"{result['_path']}: dirty worktree evidence is forbidden")
        profile = result.get("profile_id")
        target = result.get("target")
        if not isinstance(profile, str) or profile not in required:
            fail(f"{result['_path']}: unexpected profile_id {profile!r}")
        if not isinstance(target, str) or not profile.startswith(target + "/"):
            fail(f"{result['_path']}: profile/target mismatch")
        if expected_revision and result.get("git_revision") != expected_revision:
            fail(
                f"{result['_path']}: stale git_revision "
                f"{result.get('git_revision')!r} != {expected_revision!r}"
            )
        if result.get("differential_passed") is not True:
            fail(f"{result['_path']}: differential_passed is not true")
        comparison = result.get("comparison") or {}
        if comparison.get("passed") is not True or comparison.get("mismatches"):
            fail(f"{result['_path']}: comparison did not pass")
        ox_engine = str((result.get("oxiland") or {}).get("engine") or "")
        red_engine = str((result.get("redland") or {}).get("engine") or "")
        if not ox_engine.endswith("-c") or not red_engine.endswith("-c"):
            fail(
                f"{result['_path']}: C oracle engines required "
                f"(oxiland={ox_engine!r}, redland={red_engine!r})"
            )
        arts = result.get("artifacts") or {}
        if not arts.get("oxiland_library") or not arts.get("redland_library"):
            fail(f"{result['_path']}: missing native library artifact provenance")
        exec_id = result.get("execution_id")
        if not isinstance(exec_id, str) or len(exec_id) < 16:
            fail(f"{result['_path']}: missing execution_id")
        # Fan-out detection: same execution_id must not appear on different targets.
        execution_by_target[target].add(exec_id)
        by_profile[profile].append(result)

    # Cross-target fan-out: an execution_id reused across targets fails.
    seen_exec: dict[str, str] = {}
    for target, exec_ids in execution_by_target.items():
        for exec_id in exec_ids:
            prior = seen_exec.get(exec_id)
            if prior and prior != target:
                fail(
                    f"profile fan-out detected: execution_id {exec_id} used on "
                    f"{prior} and {target}"
                )
            seen_exec[exec_id] = target

    missing_profiles = sorted(required - set(by_profile))
    if missing_profiles:
        fail(f"missing raw evidence for profiles: {missing_profiles}")

    # Each profile must have its own index with matching target.
    for profile in required:
        index_name = profile.replace("/", "__") + "__index.json"
        index_path = RAW / index_name
        if not index_path.is_file():
            fail(f"missing profile index: {index_name}")
        index = json.loads(index_path.read_text(encoding="utf-8"))
        if index.get("synthetic") is True:
            fail(f"{index_name}: synthetic index forbidden")
        if index.get("profile_id") != profile:
            fail(f"{index_name}: profile_id mismatch")
        if index.get("failed", 1) != 0:
            fail(f"{index_name}: failed != 0")
        if expected_revision and index.get("git_revision") != expected_revision:
            fail(f"{index_name}: stale git revision")
        if not by_profile[profile]:
            fail(f"{profile}: no fixture results")

    covered_obligations: set[str] = set()
    for result in results:
        for obl_id in result.get("obligation_ids") or []:
            if result.get("differential_passed") is True:
                covered_obligations.add(obl_id)

    return {
        "profiles": sorted(by_profile),
        "result_count": len(results),
        "covered_obligations": sorted(covered_obligations),
    }


def validate_parity_derivation(parity: dict, covered: set[str], matrix: dict) -> None:
    if parity.get("schema_version") != 1 or parity.get("milestone") != "0.11":
        fail("parity evidence has wrong schema/milestone")
    if parity.get("synthetic") is True:
        fail("parity evidence is marked synthetic")
    if parity.get("generator") and "generate-0.10" in str(parity.get("generator")):
        fail("parity evidence must not come from the 0.10 asserted generator")
    profiles = parity.get("profiles")
    if not isinstance(profiles, list):
        fail("parity evidence profiles must be a list")
    expected = set(matrix["required_profile_ids"])
    got = {p.get("id") for p in profiles if isinstance(p, dict)}
    if got != expected:
        fail(f"parity profiles {sorted(got)} != matrix {sorted(expected)}")
    for profile in profiles:
        if profile.get("differential_passed") is not True:
            fail(f"parity profile {profile.get('id')}: differential_passed is not true")
        if profile.get("skips") or profile.get("mismatches") or profile.get("quarantined") or profile.get("deviations"):
            fail(f"parity profile {profile.get('id')}: non-empty skip/mismatch/quarantine/deviation")
        if profile.get("synthetic") is True:
            fail(f"parity profile {profile.get('id')}: synthetic")
        # Must reference raw evidence, not assert from allowlist alone.
        raw_refs = profile.get("raw_results") or profile.get("raw_evidence") or []
        if not isinstance(raw_refs, list) or not raw_refs:
            fail(f"parity profile {profile.get('id')}: missing raw_results linkage")
        derived = set(profile.get("verified_obligations") or [])
        if not derived:
            fail(f"parity profile {profile.get('id')}: no verified_obligations")
        if not derived.issubset(covered):
            fail(
                f"parity profile {profile.get('id')}: verified_obligations not "
                "subset of raw covered obligations"
            )


def validate_inventory_states(inventory: dict, covered: set[str]) -> None:
    if inventory.get("milestone") != "0.11":
        fail("inventory milestone must be 0.11")
    catalog = load_json(OBLIGATIONS)
    obl_by_id = {o["id"]: o for o in catalog["obligations"]}
    required = set(obl_by_id)
    missing = sorted(required - covered)
    if missing:
        fail(
            f"{len(missing)} obligations lack passing raw evidence "
            f"(first 10): {missing[:10]}"
        )

    for entry in inventory["entries"]:
        obligations = entry.get("obligations") or []
        if not obligations:
            fail(f"{entry['id']}: missing obligations")
        all_covered = all(o in covered for o in obligations)
        if entry.get("state") == "verified" and not all_covered:
            fail(f"{entry['id']}: verified without full obligation coverage")
        if entry.get("c_state") == "verified" and not all_covered:
            fail(f"{entry['id']}: c_state verified without full obligation coverage")
        if entry.get("state") == "excluded":
            fail(f"{entry['id']}: exclusions are forbidden in 0.11")
        if entry.get("deviations"):
            fail(f"{entry['id']}: deviations are forbidden in 0.11")


def validate_soak_fuzz(soak: dict, fuzz: dict, revision: str | None) -> None:
    if soak.get("milestone") != "0.11" or soak.get("completed") is not True:
        fail("soak record incomplete")
    if soak.get("abi_resets", -1) != 0:
        fail("soak retains ABI resets")
    if soak.get("release_blockers") != []:
        fail("soak retains release blockers")
    if revision and soak.get("git_revision") != revision:
        fail("soak git revision mismatch")

    if fuzz.get("milestone") != "0.11" or fuzz.get("findings") != []:
        fail("fuzz record wrong milestone or retains findings")
    targets = fuzz.get("targets") or []
    names = {t.get("name") for t in targets if isinstance(t, dict)}
    if names != {"rdf_parser", "c_lifecycle"}:
        fail("fuzz targets incomplete")
    for target in targets:
        name = target.get("name")
        if target.get("duration_status") in {None, "planned", "queued", "smoke"}:
            fail(
                f"fuzz target {name}: duration_status must be executed "
                "(planned/smoke-only evidence is forbidden)"
            )
        duration = target.get("duration_seconds") or 0
        if duration < 3600:
            fail(f"fuzz target {name}: duration_seconds < 3600")
        if target.get("result") not in {"pass", "clean"}:
            fail(f"fuzz target {name}: not passed")
        if target.get("findings"):
            fail(f"fuzz target {name}: retains findings")


def validate_baseline() -> None:
    baseline = load_json(BASELINE)
    if baseline.get("milestone") != "0.11":
        fail("baseline manifest milestone must be 0.11")
    headers = baseline.get("headers") or []
    if len(headers) < 10:
        fail("baseline headers incomplete")
    for header in headers:
        path = ROOT / header["path"]
        if not path.is_file():
            fail(f"baseline header missing: {header['path']}")
        if sha256_file(path) != header["sha256"]:
            fail(f"baseline header checksum mismatch: {header['path']}")
    exports = baseline.get("exports") or {}
    if not exports.get("symbols"):
        fail("baseline exports missing")


def evaluate() -> dict:
    matrix = load_json(MATRIX)
    inventory = load_json(INVENTORY)
    parity = load_json(PARITY)
    soak = load_json(SOAK)
    fuzz = load_json(FUZZ)
    validate_baseline()

    revision = parity.get("git_revision")
    results = collect_raw_results()
    raw_report = validate_raw(results, matrix, revision if isinstance(revision, str) else None)
    covered = set(raw_report["covered_obligations"])
    validate_parity_derivation(parity, covered, matrix)
    validate_inventory_states(inventory, covered)
    validate_soak_fuzz(soak, fuzz, revision if isinstance(revision, str) else None)

    # Performance profiles must exist and not be labeled synthetic.
    # Only enforce profiles whose target already has differential raw evidence,
    # plus require that every matrix performance profile is present once the
    # six-cell differential matrix is complete.
    evidenced_targets = {
        result["target"] for result in results if isinstance(result.get("target"), str)
    }
    perf_profiles = list(matrix["performance"]["profiles"])
    if set(matrix["required_profile_ids"]).issubset(
        {r.get("profile_id") for r in results}
    ):
        required_perf = perf_profiles
    else:
        required_perf = [p for p in perf_profiles if p.split("/", 1)[0] in evidenced_targets]
        if not required_perf:
            fail("no performance profiles applicable to evidenced targets")

    for profile in required_perf:
        filename = profile.replace("/", "__") + ".json"
        path = QUAL / "performance" / filename
        if not path.is_file():
            fail(f"missing performance evidence: {filename}")
        data = json.loads(path.read_text(encoding="utf-8"))
        if data.get("synthetic") is True:
            fail(f"performance evidence is synthetic: {filename}")

    # Dual-profile artifact integrity: when both release-default and
    # release-all-storage are evidenced for a target, oxiland library hashes
    # must differ if the profiles advertise different Cargo features.
    by_target_profile: dict[str, dict[str, dict]] = defaultdict(dict)
    for result in results:
        target = result.get("target")
        profile = result.get("profile_id")
        if isinstance(target, str) and isinstance(profile, str):
            by_target_profile[target][profile] = result
    for target, profiles in by_target_profile.items():
        default_id = f"{target}/release-default"
        all_id = f"{target}/release-all-storage"
        if default_id in profiles and all_id in profiles:
            h1 = (profiles[default_id].get("artifacts") or {}).get("oxiland_library_sha256")
            h2 = (profiles[all_id].get("artifacts") or {}).get("oxiland_library_sha256")
            if h1 and h2 and h1 == h2:
                # Same hash is only acceptable when the build did not actually
                # change features; matrix profiles differ by design.
                fail(
                    f"{target}: release-default and release-all-storage share "
                    "identical oxiland_library_sha256 (profile rebuild required)"
                )

    # ABI-swap evidence required when the six-cell matrix is complete.
    if set(matrix["required_profile_ids"]).issubset(
        {r.get("profile_id") for r in results}
    ):
        abi_stamp = QUAL / "0.11-abi-swap.json"
        if not abi_stamp.is_file():
            fail("missing ABI-swap evidence: compatibility/qualification/0.11-abi-swap.json")
        abi = json.loads(abi_stamp.read_text(encoding="utf-8"))
        if abi.get("passed") is not True:
            fail("ABI-swap evidence did not pass")

    return {
        "passed": True,
        "profiles": raw_report["profiles"],
        "result_count": raw_report["result_count"],
        "covered_obligations": len(covered),
    }


def main() -> int:
    try:
        report = evaluate()
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"0.11 release gate failed: {error}", file=sys.stderr)
        return 1
    print("0.11 release qualification passes")
    print(
        f"profiles={len(report['profiles'])} results={report['result_count']} "
        f"obligations={report['covered_obligations']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
