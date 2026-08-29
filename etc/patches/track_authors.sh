#!/bin/sh

sqlite3 run/hospital.db \
	'
	drop table threads;
	drop table posts;
	'
