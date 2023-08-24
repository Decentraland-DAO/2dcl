(()=>{"use strict";var e,t,o,n,r={138:(e,t,o)=>{o.a(e,(async(e,t)=>{try{var n=o(50),r=o.n(n);const a=await r()();function c(e){e!==window.ethereum&&console.error("Do you have multiple wallets installed?")}a?c(a):console.log("Please install MetaMask!"),await window.ethereum.request({method:"eth_chainId"});let s=[];function i(e){window.location.reload()}window.ethereum.on("chainChanged",i);let d=null;function u(e){s=e,0===e.length?(l.classList.remove("hidden"),h.classList.add("hidden")):e[0]!==d&&(d=e[0],w(d))}window.ethereum.request({method:"eth_accounts"}).then(u).catch((e=>{console.error(e)})),window.ethereum.on("accountsChanged",u);const l=document.querySelector(".connectButton"),m=document.querySelector(".showAccount"),h=document.querySelector("#connected");async function p(){const e=await window.ethereum.request({method:"eth_requestAccounts"}).catch((e=>{4001===e.code&&console.log("Please connect to MetaMask.")}));s=e;const t=e[0];return w(t),t}function w(e){l.classList.add("hidden"),m.innerHTML=e,h.classList.remove("hidden"),fetch("/address",{method:"POST",headers:{Accept:"application/json","Content-Type":"application/json"},body:JSON.stringify({address:e})})}l.addEventListener("click",(()=>{p()}));const f=document.querySelector(".signButton"),y=document.querySelector("#signed");document.querySelector(".signature"),f&&f.addEventListener("click",(()=>{v()}));const b=e=>Array.from(e,(e=>("0"+(255&e).toString(16)).slice(-2))).join("");async function v(){const e=new URLSearchParams(window.location.search).get("payload").replace(/\\\\n/g,"\n");try{const n=s[0];let r=new TextEncoder;const a=`0x${b(r.encode(e))}`;t=n,o=await ethereum.request({method:"personal_sign",params:[a,n]}),fetch("/signature",{method:"POST",headers:{Accept:"application/json","Content-Type":"application/json"},body:JSON.stringify({by:{address:t},signature:o})}),y.classList.remove("hidden")}catch(e){console.error(e)}var t,o}t()}catch(_){t(_)}}),1)},50:e=>{e.exports=function({mustBeMetaMask:e=!1,silent:t=!1,timeout:o=3e3}={}){!function(){if("boolean"!=typeof e)throw new Error("@metamask/detect-provider: Expected option 'mustBeMetaMask' to be a boolean.");if("boolean"!=typeof t)throw new Error("@metamask/detect-provider: Expected option 'silent' to be a boolean.");if("number"!=typeof o)throw new Error("@metamask/detect-provider: Expected option 'timeout' to be a number.")}();let n=!1;return new Promise((r=>{function a(){if(n)return;n=!0,window.removeEventListener("ethereum#initialized",a);const{ethereum:o}=window;if(!o||e&&!o.isMetaMask){const n=e&&o?"Non-MetaMask window.ethereum detected.":"Unable to detect window.ethereum.";!t&&console.error("@metamask/detect-provider:",n),r(null)}else r(o)}window.ethereum?a():(window.addEventListener("ethereum#initialized",a,{once:!0}),setTimeout((()=>{a()}),o))}))}}},a={};function c(e){var t=a[e];if(void 0!==t)return t.exports;var o=a[e]={exports:{}};return r[e](o,o.exports,c),o.exports}e="function"==typeof Symbol?Symbol("webpack queues"):"__webpack_queues__",t="function"==typeof Symbol?Symbol("webpack exports"):"__webpack_exports__",o="function"==typeof Symbol?Symbol("webpack error"):"__webpack_error__",n=e=>{e&&e.d<1&&(e.d=1,e.forEach((e=>e.r--)),e.forEach((e=>e.r--?e.r++:e())))},c.a=(r,a,c)=>{var s;c&&((s=[]).d=-1);var i,d,u,l=new Set,m=r.exports,h=new Promise(((e,t)=>{u=t,d=e}));h[t]=m,h[e]=e=>(s&&e(s),l.forEach(e),h.catch((e=>{}))),r.exports=h,a((r=>{var a;i=(r=>r.map((r=>{if(null!==r&&"object"==typeof r){if(r[e])return r;if(r.then){var a=[];a.d=0,r.then((e=>{c[t]=e,n(a)}),(e=>{c[o]=e,n(a)}));var c={};return c[e]=e=>e(a),c}}var s={};return s[e]=e=>{},s[t]=r,s})))(r);var c=()=>i.map((e=>{if(e[o])throw e[o];return e[t]})),d=new Promise((t=>{(a=()=>t(c)).r=0;var o=e=>e!==s&&!l.has(e)&&(l.add(e),e&&!e.d&&(a.r++,e.push(a)));i.map((t=>t[e](o)))}));return a.r?d:c()}),(e=>(e?u(h[o]=e):d(m),n(s)))),s&&s.d<0&&(s.d=0)},c.n=e=>{var t=e&&e.__esModule?()=>e.default:()=>e;return c.d(t,{a:t}),t},c.d=(e,t)=>{for(var o in t)c.o(t,o)&&!c.o(e,o)&&Object.defineProperty(e,o,{enumerable:!0,get:t[o]})},c.o=(e,t)=>Object.prototype.hasOwnProperty.call(e,t),c(138)})();