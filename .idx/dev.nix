# To learn more about how to use Nix to configure your environment
# see: https://firebase.google.com/docs/studio/customize-workspace
{ pkgs, ... }: {
  # Which nixpkgs channel to use.
  channel = "unstable"; # Using unstable for latest Rust toolchain with edition2024 support

  # Use https://search.nixos.org/packages to find packages
  packages = [
    pkgs.rustc          # Rust compiler
    pkgs.cargo          # Rust package manager
    pkgs.rustfmt        # Rust code formatter
    pkgs.clippy         # Rust linter
    pkgs.rust-analyzer  # Language server for IDE support
    pkgs.gcc            # C compiler (needed for some Rust dependencies)
    pkgs.pkg-config     # Package config tool (needed for some crates)
    pkgs.openssl        # SSL library (commonly needed dependency)
    pkgs.sqlite         # SQLite database (commonly used with Rust)
    pkgs.git            # Version control
  ];

  # Sets environment variables in the workspace
  env = {
    # Rust-specific environment variables
    RUST_BACKTRACE = "1";
    CARGO_HOME = "$HOME/.cargo";
    RUSTUP_HOME = "$HOME/.rustup";
  };

  idx = {
    # Search for the extensions you want on https://open-vsx.org/ and use "publisher.id"
    extensions = [
      "rust-lang.rust-analyzer"    # Official Rust language server
      "vadimcn.vscode-lldb"        # Debugger for Rust
      "serayuzgur.crates"          # Crates.io integration
      "tamasfe.even-better-toml"   # Better TOML support for Cargo.toml
    ];

    # Enable previews
    previews = {
      enable = false;
    };

    # Workspace lifecycle hooks
    workspace = {
      # Runs when a workspace is first created
      onCreate = {
        # Create a new Rust project if none exists
        rust-init = ''
          if [ ! -f Cargo.toml ]; then
            cargo init --name workspace .
          fi
        '';
      };
      # Runs when the workspace is (re)started
      onStart = {
        # Update Rust toolchain and dependencies
        rust-setup = ''
          echo "Setting up Rust development environment..."
          cargo --version
          rustc --version
        '';
      };
    };
  };
}