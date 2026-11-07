-- Array columns of one and two dimensions, over integers and modified types.
CREATE TABLE t_col_array (
    a int4[],
    b int8[][],
    c numeric(10, 2)[],
    d varchar(10)[],
    e text[][] NOT NULL
);
