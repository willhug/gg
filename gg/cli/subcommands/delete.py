from plumbum import cli

from gg.cli.gg import GG
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_delete import safe_git_delete_branch
from gg.gateways.git.branch_info import get_current_branch
from gg.lib.branch_name import get_previous_branch, get_prefix_branch_name
from gg.lib.log import logger


@GG.subcommand("del")
class GGDelete(cli.Application):
    DESCRIPTION = """
Delete branches, by default the current one (this function is really more for quick testing of branches, use at your own discretion)
"""

    branch_name = cli.SwitchAttr(['-b', '--branch'], str, help="full branch name to delete", default=None)

    def main(self, *args):
        current_branch = get_current_branch()
        if current_branch is None and self.branch_name is None:
            logger.error("need to provide a branch name or run this command off of an existing branch")
            return 1

        can_safely_delete_branch_name = self.branch_name is not None and self.branch_name != current_branch
        if can_safely_delete_branch_name:
            return self.delete_branch(self.branch_name)

        branch_to_delete = current_branch
        previous_branch = get_previous_branch(current_branch)
        if previous_branch is None:
            logger.error("cannot delete current branch, we don't know which branch to checkout instead")
            return 1

        logger.info("checking out previous branch: %s" % previous_branch)
        git_checkout(previous_branch)

        self.delete_branch(branch_to_delete)

    def delete_branch(self, branch_name: str) -> int:
        logger.info("deleting branch: '%s'" % branch_name)

        safe_git_delete_branch(branch_name)
        prefix_branch = get_prefix_branch_name(branch_name)
        safe_git_delete_branch(prefix_branch)
