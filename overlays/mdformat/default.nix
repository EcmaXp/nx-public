_: final: prev: {
  mdformat = prev.mdformat.withPlugins (ps: [ ps.mdformat-frontmatter ]);
}
