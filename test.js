const hello = "world";
console.log(hello)

let buf = [];
for(let i=0; i<1000; i++) {
  buf.push(i);
}

for (let i=0; i<buf.length; i++) {
  console.log(buf[i]);
}