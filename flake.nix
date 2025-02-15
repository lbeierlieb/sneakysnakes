{
  description = "Naersk package and dev shell for Bevy projects";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils, nixpkgs, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };

        nativeBuildInputs = with pkgs; [
          pkg-config
          rustc
          rustfmt
          cargo
          llvmPackages.lld
          wasm-bindgen-cli
          binaryen
        ];

        buildInputs = with pkgs; [
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          libxkbcommon
          wayland
        ];

        libraryPath = pkgs.lib.makeLibraryPath buildInputs;
      in
      rec {
        # For `nix build` & `nix run`:
        sneakysnakes_bevy = naersk'.buildPackage {
          src = ./.;
          inherit nativeBuildInputs;
          inherit buildInputs;
          cargoBuildOptions = x: x ++ [ "--no-default-features" ];
        };
        sneakysnakes = pkgs.stdenv.mkDerivation {
          pname = "sneakysnakes";
          version = "1.0";

          buildInputs = [ sneakysnakes_bevy ];
          src = ./.;

          installPhase = ''
            mkdir -p $out/bin
            echo '#!${pkgs.stdenv.shell}' > $out/bin/sneakysnakes
            echo 'export LD_LIBRARY_PATH=${libraryPath}:$LD_LIBRARY_PATH' >> $out/bin/sneakysnakes
            echo '${sneakysnakes_bevy}/bin/sneakysnakes' >> $out/bin/sneakysnakes
            chmod +x $out/bin/sneakysnakes
          '';
        };
        defaultPackage = sneakysnakes;

        # For `nix develop`:
        devShell = pkgs.mkShell {
          inherit nativeBuildInputs;
          inherit buildInputs;

          LD_LIBRARY_PATH = libraryPath;
        };

        # WASM
        sneakysnakes_wasm = naersk'.buildPackage {
          src = ./.;
          inherit nativeBuildInputs;
          inherit buildInputs;
          cargoBuildOptions = x: x ++ [
            "--no-default-features"
            "--target wasm32-unknown-unknown"
          ];
        };
        sneakysnakes-wasm = pkgs.stdenv.mkDerivation {
          pname = "sneakysnakes-wasm";
          version = "1.0";

          inherit nativeBuildInputs buildInputs;

          src = ./.;

          buildPhase = ''
            wasm-bindgen --out-dir wasm-app --target web ${sneakysnakes_wasm}/bin/sneakysnakes.wasm
            wasm-opt -Oz --strip-debug wasm-app/sneakysnakes_bg.wasm -o wasm-app/sneakysnakes_bg.wasm
          '';

          installPhase = ''
            mkdir $out
            cp -r wasm-app $out/wasm
            cp wasm/index.html $out/wasm
            mkdir $out/bin
            cat > $out/bin/sneakysnakes-wasm <<EOF
            #!/usr/bin/env bash
            ${pkgs.python3}/bin/python -m http.server -d $out/wasm 8000
            EOF
            chmod +x $out/bin/sneakysnakes-wasm
          '';
        };
      }
    );
}
