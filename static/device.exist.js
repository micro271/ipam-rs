import {send_data} from './main.js'

const popoverTriggerList = document.querySelectorAll('[data-bs-toggle="popover"]')
const popoverList = [...popoverTriggerList].map(popoverTriggerEl => {
    new bootstrap.Popover(popoverTriggerEl,{html: true})
    
});


const reserved_button = document.getElementById('reserve_ip');

if (reserved_button) {
    reserved_button.addEventListener('click', reserve_ip);
}

const reserve_ip = (event) => {
    console.log(event.target.parentNode)
}

[...popoverTriggerList].map(popOver => popOver.addEventListener('shown.bs.popover', () => {
    const pop_over = bootstrap.Popover.getInstance(popOver);
    const id_father = pop_over.tip.id;
    const father = document.getElementById(id_father);
    const body = father.querySelector(".popover-body");


    const status = father.querySelector("#data_status");
    const ip = father.querySelector('.popover-header p').textContent;
    const ip_ = ip.replaceAll('.','_');

    const button_reserved = body.querySelector("#to_reserve");
    const network_id = document.getElementById('network_id').textContent;

    const event_reserve = async () => {
        
        const resp = await fetch(`/api/v1/device/reserve?ip=${ip}&network_id=${network_id}`,{
            method: 'PATCH'
        });

        if (resp.ok) {
            location.reload();
        }
    }

    if (button_reserved){
        button_reserved.addEventListener('click',event_reserve)
    }

    const event_edit = () => {
        const modal = document.querySelector(".modal");
        
        const description = body.querySelector("#description");
        const user = body.querySelector("#username");
        const pass = body.querySelector("#password");
        const location = body.querySelector("#location");
        const input_address = modal.querySelector("[name='address']");
        const input_description = modal.querySelector("[name='description']");
        const input_user = modal.querySelector("[name='username']");
        const input_location = modal.querySelector("[name='location']");
        const input_pass = modal.querySelector("[name='password']");
        const checkbox_address_allow_change = modal.querySelector("#checkbox_to_change_address");
        const checkbox_location_allow_cange = modal.querySelector("#checkbox_to_change_location");

        input_address.value = ip;
        input_description.value = description.textContent;
        input_user.value = user.textContent;
        input_pass.value = pass.textContent;
        input_location.disabled = true;

        if (location.href) {
            const path = location.href.split("/");
            const id = path[path.length-1];
            input_location.value = id;
        } else {
            input_location = 'empty';
        }

        checkbox_location_allow_cange.addEventListener('change', (tg) => {
            const checkbox = tg.currentTarget;
            if (checkbox.checked) {
                input_location.disabled = false;
            } else {
                input_location.disabled = true;
            }
        });
        
        checkbox_address_allow_change.addEventListener('change', (tg) => {
            const checkbox = tg.currentTarget;
            if (checkbox.checked) {
                input_address.disabled = false;
            } else {
                input_address.disabled = true;
            }
        });
        pop_over.hide();
        new bootstrap.Modal(modal).show();

        const save = modal.querySelector(".save");
        const event_save = async () => {
            const send = {};
            if (description.textContent != input_description.value) {
                send.description = input_description.value;
            }

            if (checkbox_location_allow_cange.checked && location.textContent != input_location.value) {
                send.rack = input_location.value;
            }

            if (pass.textContent != input_pass.value || user.textContent != input_user.value) {
                send.credential = {
                    password: input_pass.value,
                    username: input_user.value,
                };
            }

            if (input_address.value != ip && checkbox_address_allow_change.checked) {
                send.ip = input_address.value;
            }
            
            if (description.textContent != input_description.value) {
                send.description = input_description.value;
            }
            
            if (Object.keys(send).length > 0) {
                const network_id = document.getElementById("network_id").textContent;

                const resp = await fetch(`/api/v1/device?ip=${ip}&network_id=${network_id}`, {
                    method: 'PATCH',
                    headers: {'Content-type':'application/json'},
                    body: JSON.stringify(send)
                })

                if (resp.ok) {
                    bootstrap.Modal.getInstance(modal).hide();
                    window.location.reload(true);
                }
            }
        }
        save.addEventListener('click', event_save);
        modal.addEventListener('hidden.bs.modal',() => modal.querySelector(`.save`).removeEventListener('click',event_save));
    }

    const buttono_edit = body.querySelector("#edit_device");
    buttono_edit.addEventListener("click", event_edit)

    popOver.addEventListener('hidden.bs.popover', () => {
        if (button_reserved) {
            button_reserved.removeEventListener('click',event_reserve);
        }
        buttono_edit.removeEventListener('click', event_edit);
    })
}));

[...document.querySelectorAll("[data-ipam-ping]")].forEach(anchor => {
    anchor.addEventListener('click',async (anchor) => {
        const button = anchor.currentTarget;
        const ip = button.getAttribute("data-ipam-ping").replaceAll("_",".");
        const network_id = document.getElementById("network_id").textContent;
        if (ip) {
            const req = await fetch(`/api/v1/device/ping?ip=${ip}&network_id=${network_id}`,{
                method: 'PATCH'
            });
            location.reload(true)
        }
    })
})

document.getElementById("walk").addEventListener("click", async (event) => {
    const button = event.currentTarget;
    if (button.getAttribute("data-ipam-walk") === 'false') {
        button.setAttribute('data-ipam-walk','true');
    } else {
        button.setAttribute('data-ipam-walk','false');
    }

    if (button.getAttribute("data-ipam-walk") === 'true') {
        button.textContent = "Stop"
        button.classList.remove("btn-primary");
        button.classList.add("btn-danger");
        const spin = document.querySelectorAll("[data-ipam-ping]");
        const network_id = document.getElementById("network_id").textContent;
        if (spin) {
            for (const btn of [... spin]) {
                if (button.getAttribute("data-ipam-walk") === 'false') {
                    break;
                }

                btn.style.animation = "spinWalk 0.6s infinite";
                btn.classList.remove("link-danger");
                btn.classList.add("link-success");
                const ip_ = btn.getAttribute("data-ipam-ping");
                const ip = ip_.replaceAll("_",".");
                const ping = await fetch(`/api/v1/device/ping?ip=${ip}&network_id=${network_id}`,{
                    method: 'PATCH'
                });
                const resp = await ping.json();
                console.log(resp);
                const svg = document.getElementById(`svg_${ip_}`);
                if (resp.ping == 'Pong' && !svg.classList.contains('svg-online')) {
                    svg.classList = "svg-online";
                } else if (resp.ping == 'Fail' && svg.classList.contains('svg-online')) {
                    svg.classList = "svg-offline";
                }
                btn.style.animation = "";
                btn.classList.remove("link-success");
                btn.classList.add("link-danger");
                
            }
            location.reload(true);
        }
    }    
})


const button_create_missing = document.getElementById("missing_devices");

if (button_create_missing) {
    button_create_missing.addEventListener('click', async (e) => {
        e.preventDefault();

        const endpoint = document.getElementById("network_id").innerHTML;

        const create_all_devices = await fetch(`/api/v1/device/${endpoint}`,{
            headers: {"Content-type": "application/json"},
            method: 'POST',
        });
        
        if (create_all_devices.ok) {
            location.reload(true)
        }
    })
}