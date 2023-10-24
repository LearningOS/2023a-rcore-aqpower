DOCKER_NAME ?= rcore-tutorial-v3
.PHONY: docker build_docker
	
CONTAINER_NAME = rCore

once_docker:
	docker run --rm -it -v ${PWD}:/mnt -w /mnt ${DOCKER_NAME} bash

build_docker: 
	docker build -t ${DOCKER_NAME} .

init_docker:
	docker run --name rCore -it -v ${PWD}:/mnt -w /mnt rcore-tutorial-v3 bash
	
docker:
	docker restart ${CONTAINER_NAME} && docker exec -it ${CONTAINER_NAME} /bin/bash -c "cd /mnt/os; bash";

fmt:
	cd os ; cargo fmt;  cd ..

