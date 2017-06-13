import json
from urllib import parse

import requests

from gg.gateways.git import REPO_USER, REPO_NAME
from gg.gateways.github import GITHUB_API_URL, GITHUB_TOKEN
from gg.gateways.github.pr_info import PullRequest
from gg.lib.log import logger


def merge_pr(pr: PullRequest, head_ref: str) -> bool:
    """Send a request to github for the core pull request info and get the result"""
    pr_query_url = "{api_url}/repos/{user}/{repo}/pulls/{number}/merge".format(
        api_url=GITHUB_API_URL,
        user=REPO_USER,
        repo=REPO_NAME,
        number=pr.core.number
    )
    headers = {
        'Authorization': 'bearer %s' % GITHUB_TOKEN,
        'Accept': 'application/vnd.github.polaris-preview'
    }
    json_data = {
        'commit_title': pr.core.title + " (#" + str(pr.core.number) + ")" or "",
        'commit_message': pr.core.body or "",
        'sha': head_ref,
        'merge_method': 'squash',
    }
    resp = requests.put(pr_query_url, json=json_data, headers=headers)

    if resp.status_code != 200:
        logger.error("invalid request for merge!")
        logger.error(resp.text)
        raise Exception("invalid request for merge!")

    resp_body = json.loads(resp.text)

    return resp_body['merged']
