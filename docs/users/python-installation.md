# Python installation and compatibility

!!! info "Release status"

    PyPI currently publishes **0.8.0**. This tip is **0.9.0** (unreleased) until
    the tag. Pin published wheels as shown below; build from a git checkout for
    tip APIs.

## Supported runtime

| Component | Support |
|---|---|
| Interpreter | CPython 3.10, 3.11, 3.12, 3.13, and 3.14 |
| Operating system | Linux, macOS, and Windows wheel targets built in CI |
| Architecture | The architecture encoded by an available PyPI wheel |
| Python dependencies | None required at runtime |
| Static typing | Bundled `py.typed` marker and `.pyi` declarations |
| Package format | Binary wheel only; no source distribution on PyPI |

Each release wheel is installed and imported on its matching Python version in
CI. Compatibility means a wheel exists for the exact Python, operating-system,
and architecture tags selected by `pip`.

## Recommended installation

Create an isolated environment and upgrade `pip` before installing:

=== "macOS / Linux"

    ```console
    python3 -m venv .venv
    source .venv/bin/activate
    python -m pip install --upgrade pip
    python -m pip install oxiland==0.8.0
    ```

=== "Windows PowerShell"

    ```powershell
    py -m venv .venv
    .venv\Scripts\Activate.ps1
    python -m pip install --upgrade pip
    python -m pip install oxiland==0.8.0
    ```

Confirm the installed runtime before deployment:

```console
python -c "import oxiland; print(oxiland.__version__)"
```

```python
import oxiland

assert tuple(map(int, oxiland.__version__.split("."))) >= (0, 8, 0)
assert oxiland.Model().backend == "memory"
```

## Reproducible application builds

Pin Oxiland in the same lock or constraints workflow as the rest of the
application:

```text
# requirements.in
oxiland==0.8.0
```

For environments that require artifact integrity, download the wheel and
`SHA256SUMS` from the matching GitHub release, verify the checksum, and retain
the GitHub build-provenance attestation with deployment evidence. You can also
generate hashes with your dependency-locking tool and install with pip hash
checking. Do not copy a wheel between platforms based only on the version
number; wheel tags include the CPython ABI, operating system, and architecture.

When building containers, resolve dependencies for the image's actual platform
and libc environment. A wheel downloaded for macOS, Windows, or a different
Linux architecture is not portable to the container.

## Diagnosing installation failures

Use verbose `pip` output to see which tags were considered:

```console
python -m pip install --verbose oxiland
python -m pip debug --verbose
```

Common causes are:

- running PyPy instead of CPython;
- using Python older than 3.10 or newer than the published matrix;
- targeting an architecture for which no wheel was published;
- using an old `pip` that does not recognize the wheel's platform tag;
- forcing source-only installation with `--no-binary`.

Because published packages are **wheels only** (no source distribution), `pip`
should not compile Oxiland as a fallback. On an unsupported target, choose a
supported runtime or build from a repository checkout.

## Building from a checkout

Source builds are a contributor and platform-porting workflow—and the way to
evaluate tip **0.9** before the PyPI tag. They require a Rust toolchain,
Maturin, and the repository source:

```console
git clone https://github.com/eddiethedean/oxiland.git
cd oxiland/python
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip maturin
maturin develop --release
python -c "import oxiland; print(oxiland.__version__)"
```

See [Contributing](../contributing.md) for the complete test and quality-check
workflow. Application users installing a published wheel do not need Maturin,
Cargo, or a Rust compiler.

## Upgrades

Read the [changelog](https://github.com/eddiethedean/oxiland/blob/main/CHANGELOG.md)
before changing minor versions. Oxiland is pre-1.0, so a minor release may
contain documented API changes. Persistent deployments should also follow the
[upgrade runbook](python-production.md#upgrade-runbook).
