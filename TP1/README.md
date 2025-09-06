# TP1 - Programación Concurrente

**Alumno**: Jonathan Dominguez

**Padrón**: 110057

**Dataset utilizado**: [Anime Dataset 2023](https://www.kaggle.com/datasets/dbdmobile/myanimelist-dataset?select=final_animedataset.csv)

Este dataset "final_animedataset.csv" recopila la informacion de los usuarios que califican animes y los datos de los mismos.

Cada set contiene los campos de:

| username          | anime_id     | my_score            | user_id        | gender              |
| ------------------- | -------------- | --------------------- | ---------------- | --------------------- |
| Nombre de usuario | ID del anime | Puntaje del usuario | ID del usuario | Género del usuario |

| tittle            | type                                | source                            | score                               | scored_by                             | rank                                              | popularity               | genre                                        |
| ------------------- | ------------------------------------- | ----------------------------------- | ------------------------------------- | --------------------------------------- | --------------------------------------------------- | -------------------------- | ---------------------------------------------- |
| Título del anime | Tipo de anime (serie, pelicula, etc) | Origen (manga, novela ligera, etc) | Puntaje promedio que tiene el anime | Cantidad de personas que lo puntuaron | En que top está el anime, siendo el 1° el mejor | La popularidad del anime | Los géneros del anime (romance, terror, etc) |

## Ejecución

```
cargo run <ruta dataset> <cantidad de threads> <nombre de salida>
```

Ejemplo de ejecución:

```
cargo run tp1/dataset/anime.csv 2 top.json
```

## Transformaciones

## Tests

## Performance

## Conclusiones
