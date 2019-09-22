-- Your SQL goes here
CREATE TABLE IF NOT EXISTS posts( 
	id INTEGER NOT NULL PRIMARY KEY, 
	content TEXT, 
	parent_id INT REFERENCES posts (id)
); 
	
	
INSERT INTO posts (content, parent_id) VALUES ("1", 0); 
INSERT INTO posts (content, parent_id) VALUES ("1.1", 1);
INSERT INTO posts (content, parent_id) VALUES ("1.2", 1);
INSERT INTO posts (content, parent_id) VALUES ("1.2.1", 2);