/* select .unix-time out of the dom */
let times=document.getElementsByClassName("unix-time");

/* index days into this array to get back the str repr of the day of the week */
let days=["sun","mon","tue","wed","thu","fri","sat"];

/* foreach time */
for (let i=0;i<times.length;i++) {
	/* for e in times */
	let e=times.item(i);
	/* (int)e */
	let t=parseInt(e.innerHTML);
	/* conv timestamp to ms */
	let d=new Date(t*1000);
	/* hour */
	let h=d.getHours();
	/* am/pm */
	let m=h>12?"pm":"am";
	/* fmt'd timestamp */
	let fmt=`${days[d.getDay()%7]}, ${d.getFullYear()}-${d.getMonth()}-${d.getDate()} ${h}:${String(d.getMinutes()).padStart(2, '0')}${m}`;
	/* set */
	e.innerHTML=fmt;
};
