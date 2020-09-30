var express = require('express');
const { createReadStream } = require('fs');
const fs = require('fs');

var PORT = process.env.PORT || 8080;
var app = express();

app.get('/stream', (req, res, next) => {
  if (req.method === 'HEAD') {
    return next();
  }
  let options = getRangeOptions(req);
  const readStream = createReadStream('./data/big.json', options);
  readStream.pipe(res);
});


app.get('/stream/timeout', (req, res, next) => {
  if (req.method === 'HEAD') {
    return next();
  }
  res.setHeader('Content-Type', 'plain');
  res.setHeader('Transfer-Encoding', 'chunked');
  setInterval(function(){
    console.log('time')
    res.write("data\n");
  }, 1000);
});

app.head('/stream/timeout', (_req, res) => {
  res.set('content-length', 1500);
  res.status(200).send();
});

app.head('/stream', (req, res) => {
  const stats = fs.statSync('./data/big.json');
  res.set('content-length', stats.size);
  res.status(200).send();
});


app.get('/stream/small', (req, res, next) => {
  if (req.method === 'HEAD') {
    return next();
  }
  let options = getRangeOptions(req);
  const readStream = createReadStream('./data/small.json', options);
  readStream.pipe(res);
});

app.head('/stream/small', (req, res) => {
  const stats = fs.statSync('./data/small.json');
  res.set('content-length', stats.size);
  res.status(200).send();
});


app.post('/topic', (_req, res) => {
  res.send({ test: 'pong' });
});


function getRangeOptions(req) {
  let options = {};
  const rangeHeader = req.headers.range;
  if (rangeHeader) {
    const range = rangeHeader.split('-');
    options.start = range[0] ? parseInt(range[0]) : null;
    options.end = range[1] ? parseInt(range[1]) : undefined;
  }
  return options;
}

app.listen(PORT, () => {
  console.log(`Example app listening on port ${PORT}!`);
});