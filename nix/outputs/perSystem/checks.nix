{
  __inputs = {
    imp-fmt.url = "github:imp-nix/imp.fmt";
    imp-fmt.inputs.nixpkgs.follows = "nixpkgs";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    imp-fmt.inputs.treefmt-nix.follows = "treefmt-nix";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  __functor =
    _:
    {
      pkgs,
      self,
      self',
      treefmt-nix,
      imp-fmt-lib,
      rust-overlay,
      rootSrc,
      ...
    }:
    let
      formatterEval = imp-fmt-lib.makeEval {
        inherit pkgs treefmt-nix;
        excludes = [
          "target/*"
          "**/target/*"
        ];
        rust.enable = true;
      };
    in
    {
      formatting = formatterEval.config.build.check self;

      # Package build implicitly runs tests via doCheck
      build = self'.packages.default;
    };
}
