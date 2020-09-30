FROM node:stretch-slim
WORKDIR /app
COPY package.json /app/
## Decompress big json
ADD ./data/big.json.tar.gz /app/data/

RUN npm install
COPY ./data/* /app/data/
COPY index.js /app/

CMD ["npm", "run", "start"]