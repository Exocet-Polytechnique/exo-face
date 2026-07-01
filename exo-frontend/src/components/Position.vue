<template>
    <div id="container">
        <h1 id="title">Position</h1>
        <div id="position">
            <p>Lat.: 0.0</p>
            <p>Long.: 0.0</p>
        </div>
    </div>
</template>

<script>

export default {
  name: 'PositionComponent',
}
</script>

<style>

#container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

#title {
    margin: 0.5vh;
    padding: 0;
    text-align: left;
    font-size: 6vh;
    color: #313239;
}

#position {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
}

#position p{
    margin: 0.5vh;
    padding: 0;
    text-align: left;
    font-size: 5vh;
    color: #313239;
}

</style>