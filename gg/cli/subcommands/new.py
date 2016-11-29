from plumbum import cli

from gg.cli.gg import GG
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.branch_create import git_checkout_new_branch
from gg.lib.branch_name import create_branch_name, parse_branch_name, get_prefix_branch_name
from gg.lib.log import logger


@GG.subcommand("new")
class GGNew(cli.Application):
    DESCRIPTION = """
Create new branches/features changes.  If feature/change/part are not passed in, we will infer them from the current branch
"""

    feature_name = cli.SwitchAttr(['-f', '--feature'], str, help="feature name (collection of changes)", default=None)
    change_name = cli.SwitchAttr(['-c', '--change'], str, help="feature change name", default=None)
    part = cli.SwitchAttr(['-p', '--part'], float, help="feature change part number (position in stack)", default=None)

    def main(self, *args):
        if not self.passes_validation():
            return 1
        if self.feature_name:
            return self.create_branch(
                feature=self.feature_name,
                part=self.part,
                change=self.change_name,
            )

        current_branch = get_current_branch()

        if current_branch is None:
            logger.error("Need to provide a feature name or run this command off of an existing branch")
            return 1

        branch_info = parse_branch_name(current_branch)

        feature = branch_info.feature

        if self.is_invalid_branch_piece(feature):
            logger.error("cannot use '-' in feature name '%s'" % feature)
            return 1

        if self.part is not None:
            part = self.part
        else:
            part = branch_info.part + 1

        return self.create_branch(
            feature=feature,
            part=part,
            change=self.change_name,
        )

    def passes_validation(self):
        if self.is_invalid_branch_piece(self.feature_name):
            logger.error("cannot use '-' in feature name: '%s'" % self.feature_name)
            return False
        if self.is_invalid_branch_piece(self.change_name):
            logger.error("cannot use '-' in change name: '%s'" % self.change_name)
            return False
        return True

    def is_invalid_branch_piece(self, value):
        return value is not None and "-" in value

    def create_branch(self, feature: str, part: float, change: str) -> int:
        branch_name = create_branch_name(feature, change, part)
        prefix_branch = get_prefix_branch_name(branch_name)
        logger.info("Creating branch: '%s' with prefix branch at '%s'" % (branch_name, prefix_branch))

        try:
            git_checkout_new_branch(prefix_branch)
        except Exception as e:
            logger.error("Could not create prefix branch, error: %s" % str(e))
            return 1

        try:
            return git_checkout_new_branch(branch_name)
        except Exception as e:
            logger.error("Could not create branch, error: %s" % str(e))
            return 1
