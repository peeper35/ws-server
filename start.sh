echo "==================================== Deployment Started ===================================="
cd /root/app
echo "===================== 1. Stop and Remove Running Container ====================="
docker rm aznxt-ws-server --force 2> /dev/null
docker image rm aznxt-ws-server 2> /dev/null
echo "===================== 2. Load New Image and Start Container ====================="
docker load -i aznxt-ws-server.tar
docker run --restart=always --name aznxt-ws-server -p 9000:9000 -d aznxt-ws-server
echo "==================================== Deployment Finished ===================================="