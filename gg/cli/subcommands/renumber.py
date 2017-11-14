from plumbum import cli

from gg.cli.gg import GG
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_delete import safe_git_delete_branch
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.branch_create import git_checkout_new_branch
from gg.lib.branch_name import create_branch_name, parse_branch_name, get_prefix_branch_name, branch_exists, Branch
from gg.lib.log import logger


@GG.subcommand("renum")
class GGRenumber(cli.Application):
    DESCRIPTION = """
Changes the part number of the current branch.
"""

    part = cli.SwitchAttr(['-p', '--part'], float, help="feature change part number (position in stack)", default=None)

    def main(self, *args):
        if self.part is None:
            logger.error("need to provide a part number")
            return 1

        current_branch = get_current_branch()

        if current_branch is None:
            logger.error("Need to run this command off of an existing branch")
            return 1

        branch_info = parse_branch_name(current_branch)

        feature = branch_info.feature

        if self.is_invalid_branch_piece(feature):
            logger.error("cannot use '-' in feature name '%s'" % feature)
            return 1

        if branch_exists(feature, self.part):
            logger.error("branch with part %s already exists" % str(self.part))
            return 1
        return self.renumber_branch(branch_info, self.part)

    def is_invalid_branch_piece(self, value):
        return value is None or "-" in value

    def renumber_branch(self, branch: Branch, newPart: float) -> int:
        old_branch_name = branch.branch_name()
        old_prefix_branch = get_prefix_branch_name(old_branch_name)

        branch.part = newPart
        new_branch_name = branch.branch_name()
        new_prefix_branch = get_prefix_branch_name(new_branch_name)

        # Create the new "part" branches
        git_checkout(old_prefix_branch)
        git_checkout_new_branch(new_prefix_branch)
        git_checkout(old_branch_name)
        git_checkout_new_branch(new_branch_name)

        # Delete old numbers
        safe_git_delete_branch(old_prefix_branch)
        safe_git_delete_branch(old_branch_name)
