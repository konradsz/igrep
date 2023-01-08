class Igrep < Formula
  version "1.0.0"
  desc "Interactive Grep"
  homepage "https://github.com/konradsz/igrep"
  url "https://github.com/konradsz/igrep/releases/download/v#{version}/igrep-v#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "9affe2ee1357c42ed2b295ed1b3dbfa9a5b2cc9797d1df85954d15d9e461c166"

  def install
    bin.install "ig"
  end
end
