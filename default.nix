{
  src,
  naerskLib,
  pkg-config,
}:
naerskLib.buildPackage {
  name = "rl";
  src = src;
  nativeBuildInputs = [pkg-config];
}
