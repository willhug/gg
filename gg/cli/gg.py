from __future__ import absolute_import

from plumbum import cli

class GG(cli.Application):
    PROGNAME = "gg"
    DESCRIPTION = "'Greater Git' is a wrapper around git with tight integrations into other tools"
    SUBCOMMAND_HELPMSG = ""

    def main(self, *args):
        if args:
            print("Unknown command %r" % (args[0],))
            print(self.help())
            return 1   # error exit code
        if not self.nested_command:           # will be ``None`` if no sub-command follows
            print("No command given")
            print(self.help())
            return 1   # error exit code
