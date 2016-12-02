from typing import List

from plumbum import cli

from gg.cli.gg import GG
from gg.gateways.git import REPO_USER, REPO_NAME
from gg.gateways.github.pr_info import get_login, get_core_prs, \
    get_pull_request_build, get_pull_request_reviews, PullRequest, PullRequestReview
from gg.lib.log import logger
from gg.lib.printable import Line, FB, W, G, R, B


@GG.subcommand("review")
class GGReview(cli.Application):
    DESCRIPTION = """
Show the current open PRs/Diffs for the repo
"""

    def main(self, *args):
        my_login = get_login()

        prs = get_core_prs(REPO_USER, REPO_NAME)

        other_prs = [pr for pr in prs if pr.author_login != my_login]
        for core_pr in other_prs:
            build = get_pull_request_build(core_pr.head_ref, REPO_USER, REPO_NAME)
            reviews = get_pull_request_reviews(core_pr.number, REPO_USER, REPO_NAME)
            pr = PullRequest(
                core=core_pr,
                build=build,
                reviews=reviews
            )
            line = Line(
                [
                    FB(W(pr.core.html_url), "<no url>"),
                    FB(R(pr.build.state), "<no build>"),
                    "%s(%s)" % (B(pr.get_review_info()), G(self.get_user_review_info(my_login, pr.reviews))),
                    FB(G(pr.core.author_login), "<no author>"),
                    pr.core.title,
                ],
            )
            logger.info(str(line))

    def get_user_review_info(self, user: str, reviews: List[PullRequestReview]) -> str:
        def get_submit_time(review: PullRequestReview):
            return review.submittedAt
        reviews.sort(key=get_submit_time)

        status = "TO_REVIEW"
        for review in reviews:
            if review.author != user:
                continue
            if review.state == "APPROVED" or review.state == "CHANGES_REQUESTED":
                status = review.state

        return status
