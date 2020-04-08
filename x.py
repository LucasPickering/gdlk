#!/usr/bin/env python3

import abc
import argparse
import glob
import http.server
import os
import socketserver
import subprocess
import sys

DB_SERVICE = "db"
API_SERVICE = "api"
API_DEV_DB = "gdlk"
API_TEST_DB = "gdlk_test"

COMMANDS = {}


def command(name, help_text):
    """
    Decorator used to register a command. Put this on a subclass of Command to
    make it usable.
    """

    def inner(cls):
        if name in COMMANDS:
            raise ValueError(
                "Cannot register '{}' under '{}'."
                " '{}' is already registered under that name.".format(
                    cls, name, COMMANDS[name]
                )
            )
        cls._NAME = name
        cls._HELP_TEXT = help_text
        COMMANDS[name] = cls()
        return cls

    return inner


class Command(metaclass=abc.ABCMeta):
    @property
    def name(self):
        return self._NAME

    @property
    def help_text(self):
        return self._HELP_TEXT

    def configure_parser(self, parser):
        pass

    @abc.abstractmethod
    def run(self, args):
        pass


def run_cmd(cmd, env=None, **kwargs):
    """
    Runs the given shell command

    Arguments:
        cmd {[str]} -- The command, as a list of arguments
    """
    print("+ {}".format(" ".join(cmd)))
    full_env = {**os.environ, **env} if env else None
    subprocess.run(cmd, check=True, env=full_env, **kwargs)


def run_in_docker_service(
    service, cmd, interactive=False, tty=True, env={}, **kwargs
):
    """
    Runs a command in the container corresponding to the given docker-compose
    service. This will turn the service into a container name, then run the cmd.
    """
    docker_args = []

    if interactive:
        docker_args.append("-i")

    if tty:
        docker_args.append("-t")

    for k, v in env.items():
        docker_args.append("-e")
        docker_args.append(f"{k}={v}")

    run_cmd(
        ["docker", "exec", *docker_args, f"gdlk_{service}_1", *cmd], **kwargs
    )


@command("migration", "Apply DB migrations through Diesel")
class Migrate(Command):
    def configure_parser(self, parser):
        parser.add_argument(
            "action",
            choices=["run", "revert", "redo", "revert-all"],
            help="The migration action to perform.",
        )

    def revert_all(self):
        # We can only revert one migration at a time, so run until it fails
        try:
            while True:
                run_in_docker_service(
                    API_SERVICE, ["diesel", "migration", "revert"]
                )
        except subprocess.CalledProcessError:
            pass

    def run(self, action):
        if action == "revert-all":
            self.revert_all()
        else:
            run_in_docker_service(API_SERVICE, ["diesel", "migration", action])


@command("seed", "Insert seed data into the DB")
class Seed(Command):

    SEED_PATH = "./api/seeds/*.sql"

    def run(self):
        seed_files = glob.glob(self.SEED_PATH)
        for file in seed_files:
            print(f"Executing {file}...")

            # Open the file and pipe it to the docker command
            with open(file) as f:
                run_in_docker_service(
                    DB_SERVICE,
                    ["psql", API_DEV_DB],
                    interactive=True,
                    tty=False,
                    stdin=f,
                )


@command("test", "Run tests on one or more crates")
class Test(Command):
    def __init__(self):
        self._CRATES = {"core": self.test_core, "api": self.test_api}

    def configure_parser(self, parser):
        crates = list(self._CRATES.keys())
        parser.add_argument(
            "crates",
            nargs="*",
            default=crates,
            help="Run tests on the given crate(s), or all crates",
        )
        parser.add_argument(
            "--debug",
            "-d",
            action="store_true",
            help="Run tests in debug mode (DEBUG=1, see debug! macro in core)",
        )
        parser.add_argument(
            "--tests",
            "-t",
            nargs="+",
            default=[],
            help="Test name(s) to pass along to cargo",
        )
        parser.add_argument(
            "--backtrace",
            "-b",
            choices=["0", "1", "full"],
            default="0",
            help="Value to use for RUST_BACKTRACE",
        )

    def test_core(self, debug, tests, backtrace):
        run_cmd(
            ["cargo", "test", "-p", "gdlk", *tests, "--", "--nocapture"],
            env={"DEBUG": str(int(debug)), "RUST_BACKTRACE": backtrace},
        )

    def test_api(self, debug, tests, backtrace):
        try:
            run_in_docker_service(DB_SERVICE, ["dropdb", API_TEST_DB])
        except subprocess.CalledProcessError:
            # If the DB doesn't exist, we don't care
            pass
        # Create DB and run startup scripts
        run_in_docker_service(DB_SERVICE, ["createdb", API_TEST_DB])
        run_in_docker_service(
            DB_SERVICE,
            [
                "sh",
                "-c",
                f"psql {API_TEST_DB} -f /docker-entrypoint-initdb.d/*",
            ],
        )

        db_url = f"postgres://root:root@db/{API_TEST_DB}"
        run_in_docker_service(
            API_SERVICE,
            ["diesel", "migration", "run", "--locked-schema"],
            env={"DEBUG": str(int(debug)), "DATABASE_URL": db_url},
        )
        try:
            run_in_docker_service(
                API_SERVICE,
                ["cargo", "test", *tests, "--", "--nocapture"],
                env={"DATABASE_URL": db_url, "RUST_BACKTRACE": backtrace},
            )
        except subprocess.CalledProcessError:
            # Ignore this exception, so we don't obfuscate the test output
            pass

    def run(self, crates, **kwargs):
        for crate in crates:
            print(f"===== Testing {crate} =====")
            self._CRATES[crate](**kwargs)


@command("doc", "Watch, build, and (optionally) serve docs for our code")
class Docs(Command):
    def configure_parser(self, parser):
        parser.add_argument(
            "--serve",
            "-s",
            action="store_true",
            help="If enabled, start an HTTP server to serve the docs",
        )
        parser.add_argument(
            "--host", default="0.0.0.0", help="The address to host the files on"
        )
        parser.add_argument(
            "--port",
            "-p",
            default=8080,
            type=int,
            help="The port to serve the files on",
        )

    def run(self, serve, host, port):
        cargo_process = subprocess.Popen(
            ["cargo", "watch", "-x", "doc --document-private-items"]
        )

        try:
            if serve:

                def Handler(*args,):
                    http.server.SimpleHTTPRequestHandler(
                        *args, directory="./target/doc/"
                    )

                with socketserver.TCPServer((host, port), Handler) as httpd:
                    print(f"Hosting docs on {host}:{port}")
                    httpd.serve_forever()
            else:
                cargo_process.communicate()
        except KeyboardInterrupt:
            pass
        finally:
            cargo_process.terminate()


def main():
    parser = argparse.ArgumentParser(
        description="Utility script for task execution"
    )
    subparsers = parser.add_subparsers(
        help="sub-command help", dest="cmd", required=True
    )

    for command in COMMANDS.values():
        subparser = subparsers.add_parser(command.name, help=command.help_text)
        subparser.set_defaults(func=command.run)
        command.configure_parser(subparser)

    args = parser.parse_args()
    argd = vars(args)
    func = argd.pop("func")
    argd.pop("cmd")  # Don't need this one
    try:
        func(**argd)
    except subprocess.CalledProcessError as e:
        print(str(e), file=sys.stderr)
        exit(e.returncode)


if __name__ == "__main__":
    main()
