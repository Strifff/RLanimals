const canvas = document.getElementById("myCanvas");
const ctx = canvas.getContext("2d");
canvas.height = canvas.width;
ctx.transform(1, 0, 0, -1, 0, canvas.height)

ctx.fillStyle = "red";

setInterval(async () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    const res = await fetch("/calc-new-state")
    const world = await res.json();
    const entries = world["entries"];
    for (i = 0; i < entries.length; i++) {
        entry = entries[i]
        if (entry.beast == "Herbivore") {
            //fov
            sAngle = (entry.dir - entry.fov/2)*Math.PI/180.0;
            eAngle = (entry.dir + entry.fov/2)*Math.PI/180.0;
            ctx.beginPath();
            ctx.arc(entry.pos_x, entry.pos_y, 100, sAngle, eAngle);
            ctx.lineTo(entry.pos_x, entry.pos_y);
            ctx.fillStyle = "rgba(0, 0, 200, 0.3)";
            ctx.fill();


            size = 5;
            ctx.fillStyle = "red";
        
        }
        ctx.beginPath();
        ctx.arc(entry.pos_x, entry.pos_y, size, 0, 2 * Math.PI);
        ctx.fill();
    }

}, 100)