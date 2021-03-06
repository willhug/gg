import json
from urllib import parse
from typing import List

import requests

from gg.gateways.github import GITHUB_API_URL, GITHUB_TOKEN
from gg.lib.log import Logger, logger

class CorePullRequest:
     number = None # type: int
     state = None # type: str
     title = None # type: str
     body = None # type: str
     html_url = None # type: str
     created_at = None # type: str
     base_ref = None # type: str
     head_ref = None # type: str
     author_login = None # type: str

     def __init__(
         self,
         number: int = None,
         state: str = None,
         title: str = None,
         body: str = None,
         html_url: str = None,
         created_at: str = None,
         base_ref: str = None,
         head_ref: str = None,
         author_login: str = None,
     ):
         self.number = number
         self.state = state
         self.title = title
         self.body = body
         self.html_url = html_url
         self.created_at = created_at
         self.base_ref = base_ref
         self.head_ref = base_ref
         self.author_login = author_login

def get_core_pull_request(branch_name: str, repo_user: str, repo_name: str) -> CorePullRequest:
    """Send a request to github for the core pull request info and get the result"""
    filter = "?head={user}:{branch}&state=all".format(
        user=repo_user,
        branch=parse.quote(branch_name)
    )
    prs = get_core_prs(repo_user, repo_name, filter=filter)
    if len(prs) != 1:
        raise Exception("Expected one pull request in github")

    return prs[0]

def get_core_prs(repo_user, repo_name, filter="") -> List[CorePullRequest]:
    """Get all the prs currently out there"""
    pr_query_url = "{api_url}/repos/{user}/{repo}/pulls{filter}".format(
        api_url=GITHUB_API_URL,
        user=repo_user,
        repo=repo_name,
        filter=filter,
    )
    headers = {'Authorization': 'bearer %s' % GITHUB_TOKEN}
    resp = requests.get(pr_query_url, headers=headers)

    resp_body = json.loads(resp.text)
    prs = []
    for pr_body in resp_body:
        prs.append(CorePullRequest(
            number=pr_body['number'],
            state=pr_body['state'],
            title=pr_body['title'],
            body=pr_body['body'],
            html_url=pr_body['html_url'],
            created_at=pr_body['created_at'],
            base_ref=pr_body['base']['ref'],
            head_ref=pr_body['head']['ref'],
            author_login=pr_body['user']['login'],
        ))
    return prs

def get_core_prs_issues(repo_user, repo_name, filter="") -> List[CorePullRequest]:
    """Get all the prs currently out there"""
    filter = "?review-requested=willhug"
    pr_query_url = "{api_url}/repos/{user}/{repo}/issues{filter}".format(
        api_url=GITHUB_API_URL,
        user=repo_user,
        repo=repo_name,
        filter=filter,
    )
    headers = {'Authorization': 'bearer %s' % GITHUB_TOKEN}
    resp = requests.get(pr_query_url, headers=headers)

    resp_body = json.loads(resp.text)
    prs = []
    for pr_body in resp_body:
        prs.append(CorePullRequest(
            number=pr_body['number'],
            state=pr_body['state'],
            title=pr_body['title'],
            body=pr_body['body'],
            html_url=pr_body['html_url'],
            created_at=pr_body['created_at'],
            base_ref=pr_body['base']['ref'],
            head_ref=pr_body['head']['ref'],
            author_login=pr_body['user']['login'],
        ))
    return prs


class PullRequestBuild:
    state = None # type: str
    total_count = None # type: int

    def __init__(
        self,
        state: str = None,
        total_count: str = None,
    ):
        self.state = state
        self.total_count = total_count

def get_pull_request_build(branch_name: str, repo_user: str, repo_name: str) -> PullRequestBuild:
    """Grabs the pull request build info from the github statuses endpoint"""
    status_query_url = "{api_url}/repos/{user}/{repo}/commits/{branch}/status".format(
        api_url=GITHUB_API_URL,
        user=repo_user,
        repo=repo_name,
        branch=parse.quote(branch_name)
    )
    headers = {'Authorization': 'bearer %s' % GITHUB_TOKEN}
    resp = requests.get(status_query_url, headers=headers)

    status_body = json.loads(resp.text)

    if "state" not in status_body:
        return PullRequestBuild(
            state="Unknown",
        )
    return PullRequestBuild(
        state=status_body['state'],
        total_count=status_body['total_count'],
    )

class PullRequestReview:
    state = None # type: str
    submittedAt = None # type: str
    author = None # type: str

    def __init__(
        self,
        state: str = None,
        submittedAt: str = None,
        author: str = None,
    ):
        self.state = state
        self.submittedAt = submittedAt
        self.author = author

def get_pull_request_reviews(pr_number: int, repo_user: str, repo_name: str) -> List[PullRequestReview]:
    """Get all the review for the pull request (this is a bit raw, we can parse the reviews to see what's going on later)"""
    graphql_query_url = "{api_url}/graphql".format(api_url=GITHUB_API_URL)
    headers = {'Authorization': 'bearer %s' % GITHUB_TOKEN}
    graphql_query = "query {repository(owner: \"%s\", name:\"%s\") {pullRequest(number: %d) {reviews(last: 30) {edges {node { author {login}, state, submittedAt}}}}}}" % (
        repo_user,
        repo_name,
        pr_number,
    )
    json_data = {
        "query": graphql_query
    }
    resp = requests.post(graphql_query_url, json=json_data, headers=headers)
    resp_json = json.loads(resp.text)

    raw_reviews = resp_json['data']['repository']['pullRequest']['reviews']['edges']
    reviews = []

    for raw_review in raw_reviews:
        reviews.append(
            PullRequestReview(
                state=raw_review['node']['state'],
                submittedAt=raw_review['node']['submittedAt'],
                author=raw_review['node']['author']['login']
            )
        )

    return reviews

class PullRequest:
    core = None # type: CorePullRequest
    build = None # type: PullRequestBuild
    reviews = [] # type: List[PullRequestReview]

    def __init__(
        self,
        core: CorePullRequest,
        build: PullRequestBuild,
        reviews: List[PullRequestReview],
    ):
        self.core = core
        self.build = build
        self.reviews = reviews

    def get_review_info(self) -> str:
        def get_submit_time(review: PullRequestReview):
            return review.submittedAt
        self.reviews.sort(key=get_submit_time)

        reviewer_status = dict()
        for review in self.reviews:
            if review.state == "APPROVED" or review.state == "CHANGES_REQUESTED":
                reviewer_status[review.author] = review.state

        final_status = "IN_REVIEW"
        for reviewer, status in reviewer_status.items():
            if status == "CHANGES_REQUESTED":
                return "CHANGES_REQUESTED"
            final_status = status
        return final_status


def get_github_pull_request_info(branch_name: str, repo_user: str, repo_name: str) -> PullRequest:
    """Call the github api to get vital branch info"""
    try:
        core_pr = get_core_pull_request(branch_name, repo_user, repo_name)
        pr_build = get_pull_request_build(branch_name, repo_user, repo_name)
        pr_reviews = get_pull_request_reviews(core_pr.number, repo_user, repo_name)

        return PullRequest(
            core=core_pr,
            build=pr_build,
            reviews=pr_reviews,
        )
    except Exception as e:
        return PullRequest(
            core=CorePullRequest(),
            build=PullRequestBuild(),
            reviews=[],
        )

def get_login() -> str:
    """Get the login information for the current user"""
    graphql_query_url = "{api_url}/graphql".format(api_url=GITHUB_API_URL)
    headers = {'Authorization': 'bearer %s' % GITHUB_TOKEN}
    json_data = {
        "query": "query {viewer {login}}"
    }
    resp = requests.post(graphql_query_url, json=json_data, headers=headers)
    resp_json = json.loads(resp.text)

    if "errors" in resp_json:
        logger.error(resp.text)
        raise Exception("Could not determine login")

    return resp_json['data']['viewer']['login']

