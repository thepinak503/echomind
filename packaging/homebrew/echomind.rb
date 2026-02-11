# Homebrew formula for echomind
class Echomind < Formula
  desc "A powerful, lightweight AI CLI tool for multiple chat APIs"
  homepage "https://github.com/thepinak503/echomind"
  license "MIT"
  head "master"

  depends_on "openssl@3"

  uses_from_macos "openssl" => "openssl@3"

  def install
    system "cargo", "build", "--release"

    bin.install "target/release/echomind"
    man1.install "echomind.1"

    bash_completion.install "docs/completions/echomind.bash" => "echomind"
    zsh_function.install "docs/completions/_echomind" => "_echomind"
    fish_function.install "docs/completions/echomind.fish" => "echomind.fish"

    prefix.install "README.md"
    prefix.install "docs/config.example.toml"
    prefix.install "docs/LICENSE"

    caveats <<~EOS
      Configuration files are created in ~/.config/echomind/
      For more information, visit: https://github.com/thepinak503/echomind
    EOS
  end

  test do
    system "echomind", "--version"
    assert_match(/echomind \d+\.\d+\.\d+/, shell_output("echomind --version"))
  end
end
