env:
	virtualenv env
	./env/bin/pip install pip-tools

install: env
	./env/bin/pip install -r requirements-test.txt

test:
	./env/bin/pytest tests

deps: requirements.txt requirements-test.txt
	./env/bin/pip-sync requirements.txt requirements-test.txt

requirements.txt:
	./env/bin/pip-compile requirements.in

requirements-test.txt:
	./env/bin/pip-compile requirements-test.in
