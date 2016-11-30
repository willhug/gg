from plumbum import cli

from gg.cli.gg import GG
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_info import get_current_branch
from gg.lib.branch_name import (
    get_branch_for_feature_part,
    get_first_branch_for_feature,
    get_next_branch,
    get_previous_branch,
    parse_branch_name,
)
from gg.lib.log import logger

@GG.subcommand("co")
class GGCheckout(cli.Application):
    DESCRIPTION = "Checkout a branch (can use the current branch for relative information)"

    getNext = cli.Flag(['-n', '--next'], help="checkout the next branch commit")
    getPrev = cli.Flag(['-p', '--prev'], help="checkout the previous branch commit")
    part = cli.SwitchAttr(['-a', '--part'], float, help="part number to check out", default=None)
    feature_name = cli.SwitchAttr(['-f', '--feature'], str, help="base branch name to check out", default=None)

    def main(self, *args):
        if self.getNext:
            current_branch = get_current_branch()
            if not current_branch:
                logger.error('no branch currently checked out, cannot checkout relative commit')
                return 1
            branch_to_checkout = get_next_branch(current_branch)
        elif self.getPrev:
            current_branch = get_current_branch()
            if not current_branch:
                logger.error('no branch currently checked out, cannot checkout relative commit')
                return 1
            branch_to_checkout = get_previous_branch(current_branch)
        elif self.part is not None and self.feature_name is not None:
            branch_to_checkout = get_branch_for_feature_part(self.feature_name, self.part)
        elif self.part is not None:
            current_branch = get_current_branch()
            if not current_branch:
                logger.error('no branch currently checked out, cannot checkout relative commit')
                return 1
            branch = parse_branch_name(current_branch)
            branch_to_checkout = get_branch_for_feature_part(branch.feature, self.part)
        elif self.feature_name is not None:
            branch_to_checkout = get_first_branch_for_feature(self.feature_name)
        else:
            logger.error('cannot determine branch to checkout')
            return 1

        return self.checkout_branch(branch_to_checkout)

    def checkout_branch(self, branch_name: str) -> int:
        if branch_name is None:
            logger.error('cannot determine branch to checkout')
            return 1

        git_checkout(branch_name)

        return 0

