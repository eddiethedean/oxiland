document$.subscribe(() => {
  document.querySelectorAll(".md-typeset a[href^='http']").forEach((link) => {
    if (link.hostname !== window.location.hostname) {
      link.setAttribute("rel", "noopener noreferrer");
    }
  });
});
