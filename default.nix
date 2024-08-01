{ libGL, libGLU, faad2, freeglut, glew, glfw, glm, xorg, rustPlatform, rustup, lib, makeWrapper }:

rustPlatform.buildRustPackage rec {
  buildInputs = [
    libGL
    libGLU
    faad2
    freeglut
    glew
    glfw
    glm
    xorg.libX11.dev
    xorg.libX11
    xorg.libXi
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXft
    xorg.libXinerama
  ];
  
  nativeBuildInputs = [
    makeWrapper
  ];

  name = "bansheefinder";
  version = "0.0.1";

  src = lib.cleanSource ./.;

  libraries = lib.makeLibraryPath [
    libGL
    libGLU
    faad2
    freeglut
    glew
    glfw
    glm
    xorg.libX11.dev
    xorg.libX11
    xorg.libXi
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXft
    xorg.libXinerama
  ];

  postInstall = ''
    wrapProgram $out/bin/bansheefinder2 --prefix LD_LIBRARY_PATH : ${libraries}
  '';

  cargoLock = {
    lockFile = ./Cargo.lock;
    allowBuiltinFetchGit = true;
  };
}
