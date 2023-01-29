class Igrep < Formula
  version "1.1.0"
  desc "Interactive Grep"
  homepage "https://github.com/konradsz/igrep"
  url "https://github.com/konradsz/igrep/releases/download/v#{version}/igrep-v#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "228616570faa857d7fb880c186b246598f7e79446356271d7e88b257cf6e8ee0"

  def install
    bin.install "ig"
  end
end
