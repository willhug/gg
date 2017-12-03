env:
	virtualenv -py3 -p python3 env
	./env/bin/pip3.6 install pip-tools

install: env
	./env/bin/pip3.6 install -r requirements-test.txt

test:
	./env/bin/pytest tests

deps: requirements.txt requirements-test.txt
	./env/bin/pip-sync requirements.txt requirements-test.txt

requirements.txt:
	./env/bin/pip-compile requirements.in

requirements-test.txt:
	./env/bin/pip-compile requirements-test.in
