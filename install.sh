#! /usr/bin/env bash

set -euo pipefail

cat <<EOF
     _           _   _   _      
 ___| |__  _   _| |_| |_| | ___ 
/ __| '_ \\| | | | __| __| |/ _ \\
\__ \\ | | | |_| | |_| |_| |  __/
|___/_| |_|\\__,_|\\__|\\__|_|\\___|
                                
https://www.shuttle.rs
https://github.com/shuttle-hq/shuttle

Please file an issue if you encounter any problems!
===================================================
EOF

if ! command -v curl &>/dev/null; then
  echo "curl not installed. Please install curl."
  exit
elif ! command -v sed &>/dev/null; then
  echo "sed not installed. Please install sed."
  exit
fi

REPO_URL="https://github.com/shuttle-hq/shuttle"
LATEST_RELEASE=$(curl -L -s -H 'Accept: application/json' "$REPO_URL/releases/latest")
# shellcheck disable=SC2001
LATEST_VERSION=$(echo "$LATEST_RELEASE" | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')

_install_linux() {
  echo "Detected Linux!"
  echo "Checking distro..."
  if (uname -a | grep -qi "Microsoft"); then
    OS="ubuntuwsl"
  elif ! command -v lsb_release &>/dev/null; then
    echo "lsb_release could not be found. Falling back to /etc/os-release"
    OS="$(grep -Po '(?<=^ID=).*$' /etc/os-release | tr '[:upper:]' '[:lower:]')" 2>/dev/null
  else
    OS=$(lsb_release -i | awk '{ print $3 }' | tr '[:upper:]' '[:lower:]')
  fi
  case "$OS" in
  "arch" | "manjarolinux" | "endeavouros")
    _install_arch_linux
    ;;
  "alpine")
    _install_alpine_linux
    ;;
  "ubuntu" | "ubuntuwsl" | "debian" | "linuxmint" | "parrot" | "kali" | "elementary" | "pop")
    # TODO: distribute .deb packages via `cargo-deb` and install them here
    _install_unsupported
    ;;
  *)
    _install_unsupported
    ;;
  esac
}

_install_arch_linux() {
  echo "Arch Linux detected!"
  if command -v pacman &>/dev/null; then
    pacman_version=$(sudo pacman -Si cargo-shuttle | sed -n 's/^Version *: \(.*\)/\1/p')
    if [[ "${pacman_version}" != "${LATEST_VERSION#v}"* ]]; then
      echo "cargo-shuttle is not updated in the repos, ping @orhun!!!"
      _install_unsupported
    else
      echo "Installing with pacman"
      sudo pacman -S cargo-shuttle
    fi
  else
    echo "Pacman not found"
    exit 1
  fi
}

_install_alpine_linux() {
  echo "Alpine Linux detected!"
  if command -v apk &>/dev/null; then
    if apk search -q cargo-shuttle; then
      echo "cargo-shuttle is not available in the testing repository. Do you want to enable the testing repository? (y/n)"
      read -r enable_testing
      if [ "$enable_testing" = "y" ]; then
        echo "@testing http://dl-cdn.alpinelinux.org/alpine/edge/testing" | tee -a /etc/apk/repositories
        apk update
      else
        _install_unsupported
        return 0
      fi
    fi
    if ! apk info cargo-shuttle; then
      echo "Installing cargo-shuttle"
      apk add cargo-shuttle@testing
    else
      apk_version=$(apk version cargo-shuttle | awk 'NR==2{print $3}')
      if [[ "${apk_version}" != "${LATEST_VERSION#v}"* ]]; then
        echo "cargo-shuttle is not updated in the testing repository, ping @orhun!!!"
        _install_unsupported
      else
        echo "cargo-shuttle is already up to date."
      fi
    fi
  else
    echo "APK (Alpine Package Keeper) not found"
    exit 1
  fi
}

# TODO: package cargo-shuttle for Homebrew
_install_mac() {
  _install_unsupported
}

_install_binary() {
  echo "Installing pre-built binary..."
  case "$OSTYPE" in
  linux*) target="x86_64-unknown-linux-musl" ;;
  darwin*) target="x86_64-apple-darwin" ;;
  *)
    echo "Cannot determine the target to install"
    exit 1
    ;;
  esac
  temp_dir=$(mktemp -d)
  pushd "$temp_dir" >/dev/null || exit 1
  curl -LO "$REPO_URL/releases/download/$LATEST_VERSION/cargo-shuttle-$LATEST_VERSION-$target.tar.gz"
  tar -xzf "cargo-shuttle-$LATEST_VERSION-$target.tar.gz"
  echo "Installing to $HOME/.cargo/bin/cargo-shuttle"
  mv "cargo-shuttle-$target-$LATEST_VERSION/cargo-shuttle" "$HOME/.cargo/bin/"
  popd >/dev/null || exit 1
  if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    echo "Add $HOME/.cargo/bin to PATH to run cargo-shuttle"
  fi
}

_install_cargo() {
  echo "Attempting install with cargo"
  if ! command -v cargo &>/dev/null; then
    echo "cargo not found! Attempting to install Rust and cargo via rustup"
    if command -v rustup &>/dev/null; then
      echo "rustup was found, but cargo wasn't. Something is up with your install"
      exit 1
    fi
    while true; do
      read -r -p "cargo not found! Do you wish to attempt to install Rust and cargo via rustup? [Y/N] " yn </dev/tty
      case $yn in
      [Yy]*)
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s
        source "$HOME/.cargo/env"
        ;;
      [Nn]*) exit ;;
      *) echo "Please answer yes or no." ;;
      esac
    done
    echo "rustup installed! Attempting cargo install"
  fi

  cargo install --locked cargo-shuttle
}

_install_unsupported() {
  echo "Installing with package manager is not supported"

  if command -v cargo-binstall &>/dev/null; then
    echo "Installing with cargo-binstall"
    cargo binstall -y --locked cargo-shuttle
    return 0
  fi

  while true; do
    read -r -p "Do you wish to attempt to install the pre-built binary? [Y/N] " yn </dev/tty
    case $yn in
    [Yy]*)
      _install_binary
      break
      ;;
    [Nn]*)
      read -r -p "Do you wish to attempt an install with 'cargo'? [Y/N] " yn </dev/tty
      case $yn in
      [Yy]*)
        echo "Installing with 'cargo'..."
        _install_cargo
        break
        ;;
      [Nn]*) exit ;;
      *) echo "Please answer yes or no." ;;
      esac
      ;;
    *) echo "Please answer yes or no." ;;
    esac
  done
}

if command -v cargo-shuttle &>/dev/null; then
  if [[ "$(cargo-shuttle -V)" = *"${LATEST_VERSION#v}" ]]; then
    echo "cargo-shuttle is already at the latest version!"
    exit
  else
    echo "Updating cargo-shuttle to $LATEST_VERSION"
  fi
fi

case "$OSTYPE" in
linux*) _install_linux ;;
darwin*) _install_mac ;;
*) _install_unsupported ;;
esac

cat <<EOF
Thanks for installing cargo-shuttle! 🚀

If you have any issues, please open an issue on GitHub or visit our Discord (https://discord.gg/shuttle)!
EOF

# vim: ts=2 sw=2 et:
