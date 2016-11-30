import json
from urllib import parse

import requests

from gg.gateways.git import REPO_USER, REPO_NAME
from gg.gateways.github import GITHUB_API_URL, GITHUB_TOKEN
from gg.gateways.github.pr_info import PullRequest


def update_pr_base(pr: PullRequest, new_base_ref: str) -> bool:
    """update the base ref for a pull request"""
    pr_query_url = "{api_url}/repos/{user}/{repo}/pulls/{number}".format(
        api_url=GITHUB_API_URL,
        user=REPO_USER,
        repo=REPO_NAME,
        number=pr.core.number
    )
    headers = {'Authorization': 'bearer %s' % GITHUB_TOKEN}
    json_data = {
        'base': new_base_ref,
    }
    resp = requests.patch(pr_query_url, json=json_data, headers=headers)

    resp_body = json.loads(resp.text)

    return resp_body['base']['ref'] == new_base_ref
