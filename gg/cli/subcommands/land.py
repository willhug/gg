from typing import List

from plumbum import cli

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
from gg.gateways.github.pr_info import get_core_pulL_request, get_github_pull_request_info, PullRequestReview
from gg.gateways.github.pull_request import create_pull_request
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
        # TODO deduplicate this with the pullrequest function that does the same thing
        prefix_branch = get_prefix_branch_name(current_branch)
        git_checkout(prefix_branch)
        git_push(prefix_branch, force=True)

        # Push the main branch
        git_checkout(current_branch)
        git_push(current_branch, force=True)


        # TODO fetch and validate
        if not self.has_existing_pr(current_branch):
            logger.error('no pr exists for this branch, what are you trying to land?')
            return 1

        pr = get_github_pull_request_info(current_branch, REPO_USER, REPO_NAME)

        if not self.is_accepted(pr.reviews):
            logger.error('diff has not been accepted, cannot land')
            return 1

        if not self.ignore_tests and pr.build.state != "success":
            logger.error('build is not successful, cannot land')
            return 1

        # update pr base
        if not update_pr_base(pr, self.onto):
            logger.error('failed to update base')
            return 1

        head_commit = get_commit(current_branch)

        if not merge_pr(pr, head_commit.hash):
            logger.error('failed to merge pr')
            return 1

        git_fetch()

        # Delete the branches
        next_branch = get_next_branch(current_branch)
        if next_branch is None:
            git_checkout('origin/'+ self.onto)
        else:
            git_checkout(next_branch)

        safe_git_delete_branch(current_branch)
        safe_git_delete_branch(prefix_branch)
        safe_git_delete_remote_branch('origin', current_branch)
        safe_git_delete_remote_branch('origin', prefix_branch)

        git_fetch()


    def is_accepted(self, reviews: List[PullRequestReview]) -> bool:
        def get_submit_time(review: PullRequestReview):
            return review.submittedAt
        reviews.sort(key=get_submit_time)

        reviewer_status = dict()
        for review in reviews:
            if review.state == "APPROVED" or review.state == "CHANGES_REQUESTED":
                reviewer_status[review.author] = review.state

        final_status = "IN_REVIEW"
        for reviewer, status in reviewer_status.items():
            if status == "CHANGES_REQUESTED":
                return False
            final_status = status
        return final_status == "APPROVED"

    def has_existing_pr(self, branch_name: str) -> bool:
        """Return true of there's already a PR for this remote branch"""
        try:
            get_core_pulL_request(branch_name, REPO_USER, REPO_NAME)
        except Exception:
            logger.info("PR doesn't exist")
            return False

        logger.info("PR already exists")
        return True
