import { h, Component, render } from 'https://unpkg.com/preact?module';
import htm from 'https://unpkg.com/htm?module';

const html = htm.bind(h);
function App (props) {
    return html`
        <div>
            ${props.cpus.map((cpu) => {
               return html`
                   <div>${cpu.toFixed(2)}% usage</div>
               `; 
            })}    
        </div>
    `;
}


setInterval(async () => {
    let response = await fetch("/api/cpus");

    if (response.status !== 200) {
        throw new Error(`Http error ! status: ${response.status}`);
    }
    let json = await response.json();

    const app = h('pre', null, JSON.stringify(json, null, 2));

    render(html`<${App} cpus=${json}></${App}>`, document.body);
    }, 1000);
