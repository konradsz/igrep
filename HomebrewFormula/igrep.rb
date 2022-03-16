class Igrep < Formula
  version "0.4.0"
  desc "Interactive Grep"
  homepage "https://github.com/konradsz/igrep"
  url "https://github.com/konradsz/igrep/releases/download/v#{version}/igrep-v#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "89b9e6ab2561dbc0b236ef41702aacd73fb78bc9492da97e1b627e2856f15202"

  def install
    bin.install "ig"
  end
end
