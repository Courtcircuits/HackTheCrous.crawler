DROP TABLE IF EXISTS restaurant_school;

CREATE TABLE restaurant_school(
    idrestaurant INT,
    idschool INT,
    CONSTRAINT fk_idrestaurant_rsch FOREIGN KEY (idrestaurant) REFERENCES restaurant(idrestaurant) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_idschool_rsch FOREIGN KEY (idschool) REFERENCES school(idschool) ON DELETE CASCADE ON UPDATE CASCADE
);
