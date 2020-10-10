from os import getenv
import os

if getenv('GITHUB_TOKEN') == None:
	print("GITHUB TOKEN should be set")
	exit(1)

from github import Github
import sys
import argparse
# from pygit2 import Repository
from mimetypes import guess_type

def mimetype(filename):
	guess = guess_type(filename)[0]
	if guess is None:
		guess = 'application/octet-stream'
	return guess

parser = argparse.ArgumentParser(description='Deploy files to release')
parser.add_argument('kind', choices=['release', 'update'])
parser.add_argument('-f', nargs='+', required=False)


args = parser.parse_args(sys.argv[1:])



github = Github(getenv('GITHUB_TOKEN'))
repo = github.get_repo('Mr-Llama-s-Wonderful-Soundboard/mlws_web')
releases = repo.get_releases()
release = None
for release_nightly in releases:
	if release_nightly.tag_name == 'nightly':
		release = release_nightly
		break


latest_commit = repo.get_branch(branch="master").commit

short_hash = latest_commit.sha[:7] + '...'
name = f'Nightly release {short_hash}'
body = f'@{latest_commit.commit.author.name}: {latest_commit.commit.message}'
if args.kind == 'release':
	if release:
		# Release is created
		# print(release.id, f'Nightly release {latest_commit.short_id}', latest_commit.message)
		release.delete_release()

	try:
		ref = repo.get_git_ref('tags/nightly')
		ref.delete()
	except:
		pass

	repo.create_git_tag('nightly', latest_commit.commit.message, latest_commit.sha, 'commit')

if args.kind == 'release' or release is None:
	release = repo.create_git_release('nightly', name, body, prerelease=True)

print('Uploading assets')
for f in args.f:
	if os.path.exists(f):
		filename = os.path.basename(f)
		release.upload_asset(f, name=filename, content_type=mimetype(filename))
		print(f, filename, mimetype(filename))
	
# 	release.upload_asset(f)

