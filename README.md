# Transf-ORM-CLI

A powerful command-line tool for transforming and managing ORM schemas with ease.

[![Security](https://github.com/Transf-ORM/Transf-ORM-CLI/actions/workflows/security.yml/badge.svg)](https://github.com/Transf-ORM/Transf-ORM-CLI/actions/workflows/security.yml)
[![Build](https://github.com/Transf-ORM/Transf-ORM-CLI/actions/workflows/build.yml/badge.svg)](https://github.com/Transf-ORM/Transf-ORM-CLI/actions/workflows/build.yml)

## 🚀 Features

- **Easy Schema Transformation** - Convert and transform ORM schemas seamlessly
- **Multi-Database Support** - Works with various database systems
- **Type-Safe** - Built with Rust for reliability and performance
- **CLI-First** - Intuitive command-line interface for automation
- **Extensible** - Plugin architecture for custom transformations

## 📋 Prerequisites

- Rust 1.70.0 or later
- Cargo

## 🔧 Installation

### From Source

```bash
git clone https://github.com/Transf-ORM/Transf-ORM-CLI.git
cd Transf-ORM-CLI
cargo build --release
```

The binary will be available at `./target/release/transf-orm-cli`

### Add to PATH

```bash
cp ./target/release/transf-orm-cli /usr/local/bin/
```

## 📖 Usage

### Basic Command

```bash
transf-orm-cli [OPTIONS] [COMMAND]
```

### Examples

```bash
# Display help
transf-orm-cli --help

# Transform a schema
transf-orm-cli transform --input schema.orm --output schema.sql

# Validate a schema
transf-orm-cli validate --file schema.orm
```

## 🎯 Configuration

Create a `transf-orm.config.toml` file in your project root:

```toml
[transform]
input_format = "orm"
output_format = "sql"
target_database = "postgresql"

[options]
pretty_print = true
validate_after_transform = true
```

## 📁 Project Structure

```
.
├── src/
│   └── main.rs
├── Cargo.toml
├── README.md
└── .github/
    ├── workflows/
    └── ISSUE_TEMPLATE/
```

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow Rust conventions (use `rustfmt`)
- Run `cargo clippy` to check for common mistakes
- Write tests for new features
- Update documentation as needed

### Reporting Issues

- Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) for bug reports
- Use the [feature request template](.github/ISSUE_TEMPLATE/feature_request.md) for feature suggestions
- Be as descriptive as possible

## 🧪 Testing

Run tests with:

```bash
cargo test
```

Run tests with verbose output:

```bash
cargo test -- --nocapture
```

## 🔒 Security

We take security seriously. Please see our [security policy](SECURITY.md) for reporting vulnerabilities.

Run security audits with:

```bash
cargo audit
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙋 Support

- 📚 Check the [documentation](docs/) for detailed guides
- 💬 Open an issue for questions or issues
- 🐛 Report bugs using the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)

## 🙏 Acknowledgments

Thanks to all contributors and the open-source community for making this project possible.
