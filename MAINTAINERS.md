# Project Maintainers & Contact Guide

This document explains who to contact for different aspects of the **Transf-ORM-CLI** project (CLI-only, no web/backend infrastructure).

## General Contact

For general questions, issues, or discussions:

- 📝 Open an [Issue](https://github.com/Transf-ORM/Transf-ORM-CLI/issues)
- 💬 Start a [Discussion](https://github.com/Transf-ORM/Transf-ORM-CLI/discussions)
- 🔒 Security issues: See [SECURITY.md](SECURITY.md)

## Components & Maintainers

### CLI Interface & User Experience

**Area**: Command parsing, arguments, flags, help text, output formatting, user interactions

| Component | Maintainer | Contact |
|-----------|-----------|---------|
| Command Parsing | TBD | |
| Arguments & Flags | TBD | |
| Help & Usage Text | TBD | |
| Output Formatting | TBD | |

**Report issues about**: Command not working, flags not recognized, help text unclear, output formatting issues

---

### Core Transformation Engine

**Area**: Schema transformation logic, conversion algorithms, ORM-specific logic

| Component | Maintainer | Contact |
|-----------|-----------|---------|
| Schema Parser | TBD | |
| Transformation Logic | TBD | |
| ORM Handling | TBD | |
| Type Conversion | TBD | |

**Report issues about**: Transformation fails, schemas not parsed correctly, conversion errors, type mismatches

---

### Database Connectors

**Area**: Database-specific drivers and logic

| Database | Maintainer | Contact |
|----------|-----------|---------|
| PostgreSQL | TBD | |
| MySQL | TBD | |
| SQLite | TBD | |
| Oracle | TBD | |

**Report issues about**: Database connection errors, driver issues, database-specific bugs

---

### Configuration & Environment

**Area**: Config files, environment variables, settings management

| Component | Maintainer | Contact |
|-----------|-----------|---------|
| Config Parser | TBD | |
| Environment Variables | TBD | |
| Settings Management | TBD | |

**Report issues about**: Config not loading, settings not applied, environment detection issues

---

### Error Handling & Logging

**Area**: Error messages, logging, debugging information

| Component | Maintainer | Contact |
|-----------|-----------|---------|
| Error Handling | TBD | |
| Logging System | TBD | |
| Error Messages | TBD | |

**Report issues about**: Unclear error messages, missing logs, poor error reporting

---

### Testing & Quality

**Area**: Unit tests, integration tests, test infrastructure, code quality

| Area | Maintainer | Contact |
|------|-----------|---------|
| Unit Tests | TBD | |
| Integration Tests | TBD | |
| Test Infrastructure | TBD | |
| Code Quality | TBD | |

**Report issues about**: Test failures, missing tests, low coverage, code quality problems

---

### Documentation

**Area**: User guide, API documentation, wiki, examples, tutorials

| Type | Maintainer | Contact |
|------|-----------|---------|
| User Guide | TBD | |
| CLI Commands Reference | TBD | |
| Examples & Tutorials | TBD | |
| Wiki | TBD | |

**Report issues about**: Outdated documentation, unclear explanations, missing examples

---

### Build & Release

**Area**: Cargo configuration, CI/CD, releases, GitHub workflows

| Area | Maintainer | Contact |
|------|-----------|---------|
| Build System | TBD | |
| CI/CD Pipeline | TBD | |
| Releases | TBD | |
| GitHub Workflows | TBD | |

**Report issues about**: Build failures, CI/CD issues, release problems, workflow errors

---

## How to Report Issues by Category

### Bug Report

If you found a bug related to a specific component:

1. **Identify the component** using the table above
2. **Check if unmaintained**: If "TBD", use general Issues
3. **@mention the maintainer** in your issue
4. **Follow this format**:

   ```
   ## Component
   [Component Name]
   
   ## Description
   [Clear description of the bug]
   
   ## Steps to Reproduce
   [Steps to reproduce the issue]
   
   ## Expected vs Actual
   [What you expected vs what happened]
   ```

### Feature Request

1. Open a [Discussion](https://github.com/Transf-ORM/Transf-ORM-CLI/discussions) in the "Ideas" category
2. Mention which component it relates to
3. Describe the use case and why it's needed

### Performance Issue

1. Create an issue with `perf` label
2. Include benchmark results if possible
3. @mention the maintainer of the relevant component

### Documentation Issue

1. Create an issue with `docs` label
2. Explain what's missing or incorrect
3. @mention the Documentation maintainer

## Escalation

If you can't reach a component maintainer:

1. Check their last activity on GitHub
2. Try the backup maintainer (if listed)
3. Open a general issue and ask for help
4. Reach out in [Discussions](https://github.com/Transf-ORM/Transf-ORM-CLI/discussions)

## Becoming a Maintainer

Interested in maintaining a component?

1. Open a [Discussion](https://github.com/Transf-ORM/Transf-ORM-CLI/discussions)
2. Show your contributions to the project
3. Express interest in maintaining a component
4. Work with existing maintainers to transition responsibility

---

**Last Updated**: 2026-03-17

**Note**: This document will be updated as team assignments are finalized. Check back regularly for updates!
