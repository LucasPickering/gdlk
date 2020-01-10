#!/usr/bin/env python3

import abc
import argparse
import subprocess


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


def get_api_container():
    """
    Gets the name of the gdlk_api Docker container
    """
    return "gdlk_api_1"


def run_cmd(cmd):
    """
    Runs the given shell command

    Arguments:
        cmd {[str]} -- The command, as a list of arguments
    """
    print("+ {}".format(" ".join(cmd)))
    subprocess.run(cmd)


def run_in_container(container, cmd):
    """
    Run a command in a Docker container
    """
    run_cmd(["docker", "exec", container, *cmd])


@command("migrate", "Apply DB migrations through Diesel")
class Migrate(Command):
    def configure_parser(self, parser):
        parser.add_argument(
            "--redo",
            "-r",
            action="store_true",
            help="Redo migrations instead of an initial run. Will drop all "
            "tables  and re-run all migrations.",
        )

    def run(self, redo):
        run_in_container(
            get_api_container(),
            ["diesel", "migration", "redo" if redo else "run"],
        )


@command("seed", "Insert seed data into the DB")
class Seed(Command):
    def run(self):
        run_in_container(get_api_container(), ["cargo", "run", "seed"])


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
    func(**argd)


if __name__ == "__main__":
    main()
