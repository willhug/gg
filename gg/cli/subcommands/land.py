from typing import List

from plumbum import cli

from gg.actions.delete import delete_force_local_branch, delete_force_remote_branch
from gg.actions.land import land_github_pr
from gg.cli.gg import GG
from gg.gateways.git import REPO_USER, REPO_NAME, IS_GITHUB
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_delete import safe_git_delete_branch
from gg.gateways.git.branch_delete_remote import safe_git_delete_remote_branch
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.commit_info import get_commit
from gg.gateways.git.fetch import git_fetch
from gg.gateways.git.push import git_push
from gg.gateways.github.merge import merge_pr
from gg.gateways.github.pr_info import get_core_pull_request, get_github_pull_request_info, PullRequestReview
from gg.gateways.github.update_pr import update_pr_base
from gg.lib.branch_name import get_prefix_branch_name, get_next_branch
from gg.lib.log import logger


@GG.subcommand("land")
class GGLand(cli.Application):
    DESCRIPTION = "Land a feature change"

    onto = cli.SwitchAttr(['-o', '--onto'], str, help="remote branch to land onto", default=None)
    ignore_tests = cli.Flag(['--ignore-tests'], help="ignore the remote tests")

    def main(self, *args):
        if not self.onto:
            logger.error('need to specify a branch to land onto')
            return 1

        if self.is_github():
            return self.land_github_pr()
        logger.error('no support for arc yet')
        return 1

    def is_github(self):
        return IS_GITHUB

    def land_github_pr(self) -> int:
        current_branch = get_current_branch()
        if not current_branch:
            logger.error('no branch currently checked out cannot land')
            return 1

        # Push the local branches to the remote (just to be sure)
        # TODO TODO add this back?
        # push_to_remote(current_branch)

        if land_github_pr(current_branch, self.onto, self.ignore_tests):
            # Land failed
            return 1

        # Delete the branches
        next_branch = get_next_branch(current_branch)
        if next_branch is None:
            git_checkout('origin/'+ self.onto)
        else:
            git_checkout(next_branch)

        delete_force_local_branch(current_branch)
        delete_force_remote_branch(current_branch)
