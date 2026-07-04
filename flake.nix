{
  description = "Wally development environment";

  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    systems.url = "github:nix-systems/default";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs =
    {
      self,
      nixpkgs,
      devenv,
      systems,
      ...
    }@inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devShells = forEachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              {
                # Most packages come pre-built with binaries provided by the official Nix binary
                # cache.
                # If you're modifying a package or using a package that's not built upstream, Nix
                # will build it from source instead of downloading a binary.
                # To prevent packages from being built more than once, devenv provides seamless
                # integration with binary caches hosted by Cachix.
                cachix.enable = true;

                languages.rust = {
                  enable = true;
                  toolchainFile = ./rust-toolchain.toml;

                  lsp.enable = false;
                };

                packages = with pkgs; [
                  protobuf

                  # macOS has a system called Gatekeeper which performs checks on binaries.
                  # Gatekeeper can cause nextest runs to be significantly slower. A typical sign of
                  # this happening is even the simplest of tests in cargo nextest run taking more
                  # than 0.2 seconds.
                  #
                  # Adding your terminal to Developer Tools will cause any processes run by it to
                  # be excluded from Gatekeeper. For optimal performance, add your terminal to
                  # Developer Tools. You may also need to run cargo clean afterwards.
                  cargo-nextest

                  cargo-llvm-cov
                ];
              }
            ];
          };
        }
      );
    };
}
