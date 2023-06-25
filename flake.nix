{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/release-23.05";
		flake-utils.url = "github:numtide/flake-utils";
	};

	outputs = { self, nixpkgs, flake-utils }:
		flake-utils.lib.eachDefaultSystem (system:
			let pkgs = import nixpkgs {
				inherit system;
			};
			aot_gzip = pkgs.rustPlatform.buildRustPackage {
				pname = "aot_gzip";
				version = "0.1.0";

				src = ./.;

				cargoLock.lockFile = ./Cargo.lock;
			}; in {
				devShell = pkgs.mkShell {
					buildInputs = with pkgs; [
						cargo
					];
				};

				packages.default = aot_gzip;
			}
		);
}
