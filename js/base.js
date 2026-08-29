let times = document.getElementsByClassName("unix-time");
let days = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];

for (let i = 0; i < times.length; i++) {
	let e = times.item(i);
	let t = parseInt(e.innerHTML);
	let d = new Date(t*1000);
	let fmt = `${days[d.getDay()%7]}, ${d.getFullYear()}-${d.getMonth()}-${d.getDate()} ${d.getHours()%12}:${d.getMinutes()}`;
	e.innerHTML = fmt;
};
