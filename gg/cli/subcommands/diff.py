from plumbum import cli

from gg.cli.gg import GG
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.diff import git_diff
from gg.lib.branch_name import get_prefix_branch_name


@GG.subcommand("diff")
class GGDiff(cli.Application):
    DESCRIPTION = "view the 'diff' for the current branch"

    def main(self, *args):
        current_branch = get_current_branch()
        if not current_branch:
            print('ERROR no branch currently checked out, cannot checkout relative commit')
            return 1
        prefix_branch = get_prefix_branch_name(current_branch)
        return git_diff(prefix_branch)
