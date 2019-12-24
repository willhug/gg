from typing import List

from plumbum import cli, colors

from gg.cli.gg import GG
from gg.gateways.git import REPO_USER, REPO_NAME
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.commit_info import get_commits
from gg.gateways.github.pr_info import get_github_pull_request_info, PullRequestReview
from gg.lib.branch_name import parse_branch_name, get_branches_for_feature, get_prefix_branch_name, get_all_features
from gg.lib.log import logger
from gg.lib.printable import Line, R, FB, W, B, G


@GG.subcommand("log")
class GGLog(cli.Application):
    DESCRIPTION = """
Log the current state of the world
"""
    all = cli.Flag(['-a', '--all'], help="log all the available gg branches (not just the current one)")

    def main(self, *args):
        if self.all:
            return self.print_all_features()

        self.print_current_feature()

    def print_all_features(self):
        all_features = get_all_features()
        for feature in all_features:
            branch_names = get_branches_for_feature(feature)
            title = "{feature_name} {num_branches}".format(
                feature_name= (colors.green | feature),
                num_branches= (colors.red | "<%d branches>" % len(branch_names)),
            )
            logger.info(title)

            self.print_pr_info(branch_names)
            logger.info("")


    def print_current_feature(self):
        current_branch = get_current_branch()
        branch = parse_branch_name(branch_name=current_branch)
        branch_names = get_branches_for_feature(branch.feature)

        self.print_pr_info(branch_names)

    def print_pr_info(self, branch_names):
        for branch_name in branch_names:
            self.print_branch(branch_name)

    def print_branch(self, branch_name: str):
        pr = get_github_pull_request_info(branch_name, REPO_USER, REPO_NAME)
        commits = get_commits(start_ref=get_prefix_branch_name(branch_name), end_ref=branch_name)

        line = Line(
            [
                FB(W(pr.core.html_url), "<no url>"),
                FB(R(pr.build.state), "<no build>"),
                FB(B(pr.core.state), "<no pr state>"),
                FB(B(pr.get_review_info()), "<no reviews>"),
                G(branch_name),
                FB(pr.core.title, commits[0].title if len(commits) > 0 else "<no commits>")
            ],
        )

        logger.info(str(line))

    def get_review_info(self, reviews: List[PullRequestReview]) -> str:
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
                return "CHANGES_REQUESTED"
            final_status = status
        return final_status
