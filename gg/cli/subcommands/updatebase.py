from plumbum import cli

from gg.actions.land import has_existing_pr
from gg.cli.gg import GG
from gg.gateways.git import REPO_USER, REPO_NAME, IS_GITHUB
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.github.pr_info import get_github_pull_request_info
from gg.gateways.github.update_pr import update_pr_base
from gg.lib.log import logger


@GG.subcommand("updatebase")
class GGUpdateBase(cli.Application):
    DESCRIPTION = "Land a feature change"

    onto = cli.SwitchAttr(['-o', '--onto'], str, help="remote branch to land onto", default=None)

    def main(self, *args):
        if not self.onto:
            logger.error('need to specify a branch to update to')
            return 1

        if self.is_github():
            return self.update_base()
        logger.error('no support for arc yet')
        return 1

    def is_github(self):
        return IS_GITHUB

    def update_base(self) -> int:
        current_branch = get_current_branch()
        if not current_branch:
            logger.error('no branch currently checked out cannot land')
            return 1

        if not has_existing_pr(current_branch):
            logger.error('no pr exists for this branch, what are you trying to land?')
            return 1

        pr = get_github_pull_request_info(current_branch, REPO_USER, REPO_NAME)

        # update pr base
        if not update_pr_base(pr, self.onto):
            logger.error('failed to update base')
            return 1
