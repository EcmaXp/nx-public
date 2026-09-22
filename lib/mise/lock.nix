{ lib, configFiles }:
let
  configNames = map builtins.baseNameOf configFiles;
  toolFiles = lib.filter (
    file: (builtins.fromTOML (builtins.readFile file)).tools or { } != { }
  ) configFiles;
  lockFiles = map (
    file: builtins.toPath (lib.removeSuffix ".toml" (toString file) + ".lock")
  ) toolFiles;
  locks = map (
    file:
    if builtins.pathExists file then
      builtins.fromTOML (builtins.readFile file)
    else
      throw "mise lockfile missing: ${file}"
  ) lockFiles;
  toolNames = lib.concatMap (lock: builtins.attrNames (lock.tools or { })) locks;
  mergeLock =
    prefix: left: right:
    lib.zipAttrsWith
      (
        name: values:
        let
          first = builtins.head values;
          last = lib.last values;
        in
        if builtins.length values == 1 then
          first
        else if builtins.isAttrs first && builtins.isAttrs last then
          mergeLock "${prefix}.${name}" first last
        else if first == last then
          first
        else
          throw "conflicting lock metadata: ${prefix}.${name}"
      )
      [
        left
        right
      ];
  mergedLock =
    if builtins.length (lib.unique toolNames) != builtins.length toolNames then
      throw "mise lockfiles must declare unique tools"
    else if builtins.length (lib.unique (map (lock: lock.lockfile_version or 0) locks)) > 1 then
      throw "cannot combine different mise lockfile formats"
    else
      lib.foldl' (mergeLock "mise.lock") { } locks;
in
{
  inherit configNames lockFiles mergedLock;
}
