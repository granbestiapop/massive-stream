const dummyjson = require('dummy-json');
const template = `{"topic": "{{firstName}}","status": "{{int 18 65}}", "lorem":"{{lorem min=10 max=50}}"}`;

for (i=0; i<1000000; i++){
  const result = dummyjson.parse(template);
  console.log(result);
}

