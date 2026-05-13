Name:           echomind
Version:        0.3.2
Release:        1%{?dist}
Summary:        A powerful AI CLI tool for multiple chat APIs
License:        MIT
URL:            https://github.com/thepinak503/echomind
Source0:        %{url}/archive/v%{version}/echomind-%{version}.tar.gz

BuildRequires:  rust, cargo, pkg-config, openssl-devel, gcc
Requires:       openssl-libs, ca-certificates

%description
Echomind is a command-line interface for AI chat APIs including OpenAI, Claude,
Gemini, Ollama, Grok, Mistral, Cohere, and ChatAnywhere. It features streaming
responses, interactive mode, and advanced options for temperature, tokens, and
model selection.

%prep
%autosetup -n
%setup -q -n echomind-%{version}

%build
export RUSTFLAGS="-C opt-level=3 -C lto=fat -C codegen-units=1"
cargo build --release

%install
# Install binary
install -Dm755 target/release/echomind %{buildroot}%{_bindir}/echomind

# Install documentation
install -dm755 %{buildroot}%{_docdir}/echomind-%{version}
install -m644 README.md %{buildroot}%{_docdir}/echomind-%{version}/README.md
install -m644 docs/config.example.toml %{buildroot}%{_docdir}/echomind-%{version}/config.example.toml
install -m644 docs/CHANGELOG.md %{buildroot}%{_docdir}/echomind-%{version}/CHANGELOG.md
install -m644 docs/RELEASE_NOTES.md %{buildroot}%{_docdir}/echomind-%{version}/RELEASE_NOTES.md
install -m644 docs/LICENSE %{buildroot}%{_docdir}/echomind-%{version}/LICENSE

# Install man page
install -dm755 %{buildroot}%{_mandir}/man1
install -m644 echomind.1 %{buildroot}%{_mandir}/man1/echomind.1

# Install shell completions
install -dm755 %{buildroot}%{_datadir}/bash-completion/completions
install -m644 docs/completions/echomind.bash %{buildroot}%{_datadir}/bash-completion/completions/echomind

install -dm755 %{buildroot}%{_datadir}/zsh/site-functions
install -m644 docs/completions/_echomind %{buildroot}%{_datadir}/zsh/site-functions/_echomind

install -dm755 %{buildroot}%{_datadir}/fish/vendor_completions.d
install -m644 docs/completions/echomind.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/echomind.fish

%files
%{_bindir}/echomind
%{_docdir}/echomind-%{version}
%{_mandir}/man1/echomind.1*
%{_datadir}/bash-completion/completions/echomind.bash
%{_datadir}/zsh/site-functions/_echomind
%{_datadir}/fish/vendor_completions.d/echomind.fish
%license LICENSE

%changelog
* Wed Feb 07 2025 Pinak Dhabu - 0.3.2-1
- Initial RPM packaging
- Support for all features (voice, images, PDF)
- Shell completions included
