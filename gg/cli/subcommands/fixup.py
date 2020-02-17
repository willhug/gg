from plumbum import cli, colors

from gg.cli.gg import GG
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.rebase import git_rebase_interactive, git_rebase_commit_order
from gg.lib.branch_name import get_prefix_branch_name


@GG.subcommand("fixup")
class GGFixup(cli.Application):
    DESCRIPTION = """
Fixup commits on the stack.
"""
    time = cli.Flag(['-t', '--timefix'], help="fix the timings on the branch")

    def main(self, *args):

        current_branch = get_current_branch()
        prefix_branch = get_prefix_branch_name(current_branch)
        if self.time:
            return git_rebase_commit_order(prefix_branch)

        return git_rebase_interactive(prefix_branch)
