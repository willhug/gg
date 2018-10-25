from typing import List

from gg.gateways.git import REPO_USER, REPO_NAME
from gg.gateways.git.commit_info import get_commit
from gg.gateways.git.fetch import git_fetch
from gg.gateways.github.merge import merge_pr
from gg.gateways.github.pr_info import get_core_pull_request, get_github_pull_request_info, PullRequestReview
from gg.gateways.github.update_pr import update_pr_base
from gg.lib.log import logger


def land_github_pr(branch: str, onto: str, ignore_tests: str = False) -> int:
    if not has_existing_pr(branch):
        logger.error('no pr exists for this branch, what are you trying to land?')
        return 1

    pr = get_github_pull_request_info(branch, REPO_USER, REPO_NAME)

    if not is_accepted(pr.reviews):
        logger.error('diff has not been accepted, cannot land')
        return 1

    if not ignore_tests and pr.build.state != "success":
        logger.error('build is not successful, cannot land')
        return 1

    # update pr base
    if not update_pr_base(pr, onto):
        logger.error('failed to update base')
        return 1

    head_commit = get_commit(branch)

    if not merge_pr(pr, head_commit.hash):
        logger.error('failed to merge pr')
        return 1

    git_fetch()

    return 0


def is_accepted(reviews: List[PullRequestReview]) -> bool:
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


def has_existing_pr(branch_name: str) -> bool:
    """Return true of there's already a PR for this remote branch"""
    try:
        get_core_pull_request(branch_name, REPO_USER, REPO_NAME)
    except Exception:
        logger.info("PR doesn't exist")
        return False

    logger.info("PR already exists")
    return True
