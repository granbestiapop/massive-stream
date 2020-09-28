var express = require('express');
const { createReadStream } = require('fs');

var PORT = process.env.PORT || 8080;
var app = express();

app.get('/stream', (_req, res) => {
  const readStream = createReadStream('./data/big.txt');
  readStream.pipe(res);
});


app.post('/topic', (_req, res) => {
  res.send({ test: 'pong' });
});


app.listen(PORT, () => {
  console.log(`Example app listening on port ${PORT}!`);
});