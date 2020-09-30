var express = require('express');
const { createReadStream } = require('fs');

var PORT = process.env.PORT || 8080;
var app = express();

app.get('/stream', (req, res) => {
  let options = getRangeOptions(req);
  const readStream = createReadStream('./data/big.json', options);
  readStream.pipe(res);
});

app.get('/stream/small', (req, res) => {
  let options = getRangeOptions(req);
  const readStream = createReadStream('./data/small.json', options);
  readStream.pipe(res);
});


app.post('/topic', (_req, res) => {
  res.send({ test: 'pong' });
});


function getRangeOptions(req){
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