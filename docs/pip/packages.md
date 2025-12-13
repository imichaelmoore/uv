# Managing packages

## Installing a package

To install a package into the virtual environment, e.g., Flask:

```console
$ uv pip install flask
```

To install a package with optional dependencies enabled, e.g., Flask with the "dotenv" extra:

```console
$ uv pip install "flask[dotenv]"
```

To install multiple packages, e.g., Flask and Ruff:

```console
$ uv pip install flask ruff
```

To install a package with a constraint, e.g., Ruff v0.2.0 or newer:

```console
$ uv pip install 'ruff>=0.2.0'
```

To install a package at a specific version, e.g., Ruff v0.3.0:

```console
$ uv pip install 'ruff==0.3.0'
```

To install a package from the disk:

```console
$ uv pip install "ruff @ ./projects/ruff"
```

To install a package from GitHub:

```console
$ uv pip install "git+https://github.com/astral-sh/ruff"
```

To install a package from GitHub at a specific reference:

```console
$ # Install a tag
$ uv pip install "git+https://github.com/astral-sh/ruff@v0.2.0"

$ # Install a commit
$ uv pip install "git+https://github.com/astral-sh/ruff@1fadefa67b26508cc59cf38e6130bde2243c929d"

$ # Install a branch
$ uv pip install "git+https://github.com/astral-sh/ruff@main"
```

See the [Git authentication](../concepts/authentication/git.md) documentation for installation from
a private repository.

## Editable packages

Editable packages do not need to be reinstalled for changes to their source code to be active.

To install the current project as an editable package

```console
$ uv pip install -e .
```

To install a project in another directory as an editable package:

```console
$ uv pip install -e "ruff @ ./project/ruff"
```

## Installing packages from files

Multiple packages can be installed at once from standard file formats.

Install from a `requirements.txt` file:

```console
$ uv pip install -r requirements.txt
```

See the [`uv pip compile`](./compile.md) documentation for more information on `requirements.txt`
files.

Install from a `pyproject.toml` file:

```console
$ uv pip install -r pyproject.toml
```

Install from a `pyproject.toml` file with optional dependencies enabled, e.g., the "foo" extra:

```console
$ uv pip install -r pyproject.toml --extra foo
```

Install from a `pyproject.toml` file with all optional dependencies enabled:

```console
$ uv pip install -r pyproject.toml --all-extras
```

To install dependency groups in the current project directory's `pyproject.toml`, for example the
group `foo`:

```console
$ uv pip install --group foo
```

To specify the project directory where groups should be sourced from:

```console
$ uv pip install --project some/path/ --group foo --group bar
```

Alternatively, you can specify a path to a `pyproject.toml` for each group:

```console
$ uv pip install --group some/path/pyproject.toml:foo --group other/pyproject.toml:bar
```

!!! note

    As in pip, `--group` flags do not apply to other sources specified with flags like `-r` or `-e`.
    For instance, `uv pip install -r some/path/pyproject.toml --group foo` sources `foo`
    from `./pyproject.toml` and **not** `some/path/pyproject.toml`.

## Uninstalling a package

To uninstall a package, e.g., Flask:

```console
$ uv pip uninstall flask
```

To uninstall multiple packages, e.g., Flask and Ruff:

```console
$ uv pip uninstall flask ruff
```

## Downloading packages

The `uv pip download` command downloads packages and their dependencies to a specified directory
without installing them. This is useful for creating offline package repositories or for
pre-downloading packages.

To download a package, e.g., Flask:

```console
$ uv pip download flask
```

By default, packages are downloaded to the current directory. To specify a destination directory:

```console
$ uv pip download flask -d ./packages
```

To download packages from a requirements file:

```console
$ uv pip download -r requirements.txt -d ./packages
```

To download only the specified packages without their dependencies:

```console
$ uv pip download flask --no-deps -d ./packages
```

To download packages with a constraint:

```console
$ uv pip download flask -c constraints.txt -d ./packages
```

To download only pre-built wheels (no source distributions):

```console
$ uv pip download flask --only-binary :all: -d ./packages
```

To download packages for a specific platform (e.g., for cross-platform packaging):

```console
$ uv pip download flask --python-platform manylinux_2_17_x86_64 -d ./packages
```

!!! note

    When using `--python-platform`, the downloaded wheels may not be compatible with the current
    platform, as they are selected for the target platform.
